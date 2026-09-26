use std::{
    collections::{HashMap, HashSet},
    fs, io,
    path::{Path, PathBuf},
};

use proc_macro2::Span;
use syn::{
    Attribute, Expr, ExprCall, FnArg, ImplItem, Item, ItemFn, Lit, Local, Member, Pat,
    PathArguments, Stmt, Type, UseTree,
    spanned::Spanned,
    visit::{self, Visit},
};

use crate::{commands::CommandResult, project::Project};

const SHELL_EXECUTION: &str = "RUSTY-AUDIT-001";
const DYNAMIC_SQL: &str = "RUSTY-AUDIT-002";
const PUBLIC_INPUT_AUDITED: &str = "RUSTY-AUDIT-010";
const PUBLIC_INPUT_UNRECOGNIZED: &str = "RUSTY-AUDIT-011";
const PERSISTENCE_FLOW_AUDITED: &str = "RUSTY-AUDIT-012";
const PERSISTENCE_FLOW_UNRECOGNIZED: &str = "RUSTY-AUDIT-013";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Severity {
    Error,
    Warning,
    Info,
}

impl Severity {
    fn label(self) -> &'static str {
        match self {
            Self::Error => "ERROR",
            Self::Warning => "WARN",
            Self::Info => "INFO",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Finding {
    severity: Severity,
    rule_id: &'static str,
    path: PathBuf,
    line: usize,
    column: usize,
    message: &'static str,
    detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProtectionBoundary {
    ValidatedDomainInput,
    UuidParse,
}

impl ProtectionBoundary {
    fn label(self) -> &'static str {
        match self {
            Self::ValidatedDomainInput => "validated domain input",
            Self::UuidParse => "UUID parser",
        }
    }
}

struct ProcedureInput {
    name: String,
    span: Span,
    boundary: Option<ProtectionBoundary>,
}

struct ProcedureAnalysis {
    inputs: Vec<ProcedureInput>,
    persistence_sinks: Vec<PersistenceSinkFlow>,
}

struct ProtectedBinding {
    boundary: ProtectionBoundary,
    sources: HashSet<String>,
}

struct PersistenceSinkFlow {
    name: &'static str,
    span: Span,
    arguments: Vec<PersistenceSinkArgument>,
}

struct PersistenceSinkArgument {
    role: &'static str,
    binding: Option<String>,
    boundary: Option<ProtectionBoundary>,
    sources: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct SymbolicSource {
    root: String,
    fields: Vec<String>,
}

#[derive(Debug, Clone)]
struct SymbolicProtection {
    source: SymbolicSource,
    boundary: ProtectionBoundary,
}

#[derive(Debug, Clone)]
struct FunctionSummary {
    parameters: Vec<String>,
    protected_sources: Vec<SymbolicProtection>,
    persistence_sinks: Vec<FunctionSinkFlow>,
}

#[derive(Debug, Clone)]
struct FunctionSinkFlow {
    name: &'static str,
    arguments: Vec<FunctionSinkArgument>,
}

#[derive(Debug, Clone)]
struct FunctionSinkArgument {
    role: &'static str,
    binding: Option<String>,
    boundary: Option<ProtectionBoundary>,
    sources: Vec<SymbolicSource>,
}

#[derive(Debug, Clone)]
struct ConstructorSummary {
    field_arguments: HashMap<String, usize>,
}

#[derive(Default)]
struct ProjectAuditIndex {
    functions: HashMap<String, Vec<FunctionSummary>>,
    constructors: HashMap<String, Vec<ConstructorSummary>>,
    aliases: HashMap<PathBuf, HashMap<String, String>>,
}

impl ProjectAuditIndex {
    fn index_file(&mut self, path: &Path, file: &syn::File) {
        self.aliases
            .insert(path.to_path_buf(), collect_use_aliases(file));

        for item in &file.items {
            match item {
                Item::Fn(function) if !has_procedure_attribute(&function.attrs) => {
                    if let Some(summary) = summarize_function(function) {
                        self.functions
                            .entry(function.sig.ident.to_string())
                            .or_default()
                            .push(summary);
                    }
                }

                Item::Impl(item_impl) => {
                    let Some(owner) = impl_owner_name(&item_impl.self_ty) else {
                        continue;
                    };

                    for item in &item_impl.items {
                        let ImplItem::Fn(method) = item else {
                            continue;
                        };

                        let Some(summary) = summarize_constructor(&owner, method) else {
                            continue;
                        };

                        let key = format!("{owner}::{}", method.sig.ident);

                        self.constructors.entry(key).or_default().push(summary);
                    }
                }

                _ => {}
            }
        }
    }

    fn resolve_function(&self, path: &Path, local_name: &str) -> Option<&FunctionSummary> {
        let target = self
            .aliases
            .get(path)
            .and_then(|aliases| aliases.get(local_name))
            .map(String::as_str)
            .unwrap_or(local_name);

        let candidates = self.functions.get(target)?;

        if candidates.len() == 1 {
            candidates.first()
        } else {
            None
        }
    }

    fn resolve_constructor(&self, owner: &str, method: &str) -> Option<&ConstructorSummary> {
        let key = format!("{owner}::{method}");
        let candidates = self.constructors.get(&key)?;

        if candidates.len() == 1 {
            candidates.first()
        } else {
            None
        }
    }
}

pub(crate) fn run() -> CommandResult {
    let project = Project::discover()?;
    let src = project.root().join("src");

    let mut rust_files = Vec::new();
    collect_rust_files(&src, &mut rust_files)?;

    rust_files.sort();

    let mut project_index = ProjectAuditIndex::default();

    for path in &rust_files {
        let source = fs::read_to_string(path)?;
        let relative = path.strip_prefix(project.root()).unwrap_or(path);

        let file = syn::parse_file(&source).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("could not parse {}: {error}", relative.display()),
            )
        })?;

        project_index.index_file(relative, &file);
    }

    let mut findings = Vec::new();

    for path in &rust_files {
        let source = fs::read_to_string(path)?;
        let relative = path.strip_prefix(project.root()).unwrap_or(path);

        findings.extend(
            scan_source_with_index(relative, &source, Some(&project_index)).map_err(|error| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("could not parse {}: {error}", relative.display()),
                )
            })?,
        );
    }

    findings.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then(left.line.cmp(&right.line))
            .then(left.column.cmp(&right.column))
            .then(left.rule_id.cmp(right.rule_id))
    });

    println!("Rusty audit");
    println!();
    println!("scanned : {} Rust files", rust_files.len());

    if findings.is_empty() {
        println!("result  : clean");
        return Ok(());
    }

    println!("findings: {}", findings.len());
    println!();

    for finding in &findings {
        println!(
            "{} [{}] {}:{}:{}",
            finding.severity.label(),
            finding.rule_id,
            finding.path.display(),
            finding.line,
            finding.column,
        );
        println!("  {}", finding.message);
        println!("  {}", finding.detail);
        println!();
    }

    let error_count = findings
        .iter()
        .filter(|finding| finding.severity == Severity::Error)
        .count();

    let warning_count = findings
        .iter()
        .filter(|finding| finding.severity == Severity::Warning)
        .count();

    let info_count = findings
        .iter()
        .filter(|finding| finding.severity == Severity::Info)
        .count();

    println!(
        "summary : {error_count} error(s), \
         {warning_count} warning(s), \
         {info_count} info"
    );

    if error_count > 0 {
        return Err(io::Error::other(format!("audit failed with {error_count} error(s)")).into());
    }

    Ok(())
}

fn collect_rust_files(directory: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            collect_rust_files(&path, files)?;
            continue;
        }

        if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }

    Ok(())
}

#[cfg(test)]
fn scan_source(path: &Path, source: &str) -> syn::Result<Vec<Finding>> {
    scan_source_with_index(path, source, None)
}

fn scan_source_with_index(
    path: &Path,
    source: &str,
    project_index: Option<&ProjectAuditIndex>,
) -> syn::Result<Vec<Finding>> {
    let file = syn::parse_file(source)?;

    let mut visitor = AuditVisitor {
        path,
        project_index,
        findings: Vec::new(),
    };

    visitor.visit_file(&file);

    Ok(visitor.findings)
}

struct AuditVisitor<'a> {
    path: &'a Path,
    project_index: Option<&'a ProjectAuditIndex>,
    findings: Vec<Finding>,
}

impl AuditVisitor<'_> {
    fn report(
        &mut self,
        severity: Severity,
        rule_id: &'static str,
        span: Span,
        message: &'static str,
        detail: String,
    ) {
        let start = span.start();

        self.findings.push(Finding {
            severity,
            rule_id,
            path: self.path.to_path_buf(),
            line: start.line,
            column: start.column + 1,
            message,
            detail,
        });
    }

    fn audit_procedure(&mut self, function: &ItemFn) {
        let Some(analysis) = analyze_procedure(function, self.path, self.project_index) else {
            return;
        };

        let ProcedureAnalysis {
            inputs,
            persistence_sinks,
        } = analysis;

        if inputs.is_empty() {
            return;
        }

        let summary = inputs
            .iter()
            .map(|input| {
                let status = input
                    .boundary
                    .map(ProtectionBoundary::label)
                    .unwrap_or("unrecognized");

                format!("{}: String -> {status}", input.name)
            })
            .collect::<Vec<_>>()
            .join(", ");

        self.report(
            Severity::Info,
            PUBLIC_INPUT_AUDITED,
            function.sig.ident.span(),
            "public procedure input boundaries audited",
            format!("procedure `{}`: {summary}", function.sig.ident),
        );

        for input in &inputs {
            if input.boundary.is_some() {
                continue;
            }

            self.report(
                Severity::Warning,
                PUBLIC_INPUT_UNRECOGNIZED,
                input.span,
                "public String input has no recognized protection boundary",
                format!(
                    "procedure `{}` input `{}` is externally supplied; \
                     Rusty did not recognize a validated domain input or \
                     approved parser boundary",
                    function.sig.ident, input.name,
                ),
            );
        }

        for sink in persistence_sinks {
            let traced_arguments: Vec<_> = sink
                .arguments
                .iter()
                .filter(|argument| !argument.sources.is_empty())
                .collect();

            if traced_arguments.is_empty() {
                continue;
            }

            let detail = sink
                .arguments
                .iter()
                .map(format_sink_argument)
                .collect::<Vec<_>>()
                .join(", ");

            let has_unprotected_public_flow = traced_arguments
                .iter()
                .any(|argument| argument.boundary.is_none());

            if has_unprotected_public_flow {
                self.report(
                    Severity::Warning,
                    PERSISTENCE_FLOW_UNRECOGNIZED,
                    sink.span,
                    "persistence sink receives public input without a recognized protection boundary",
                    format!(
                        "procedure `{}` -> {}: {detail}",
                        function.sig.ident,
                        sink.name,
                    ),
                );
            } else {
                self.report(
                    Severity::Info,
                    PERSISTENCE_FLOW_AUDITED,
                    sink.span,
                    "persistence sink public-input flow audited",
                    format!(
                        "procedure `{}` -> {}: {detail}",
                        function.sig.ident, sink.name,
                    ),
                );
            }
        }
    }
}

impl<'ast> Visit<'ast> for AuditVisitor<'_> {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        self.audit_procedure(node);
        visit::visit_item_fn(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        if let Some(shell) = shell_interpreter(node) {
            self.report(
                Severity::Error,
                SHELL_EXECUTION,
                node.span(),
                "shell interpreter execution detected",
                format!(
                    "Command::new({shell:?}) violates the application \
                     policy against shell interpretation"
                ),
            );
        }

        if let Some(function) = dynamic_sql_function(node) {
            self.report(
                Severity::Error,
                DYNAMIC_SQL,
                node.span(),
                "dynamic SQL construction detected",
                format!(
                    "{function} receives format!(...) output; \
                     use static SQL with bound parameters instead"
                ),
            );
        }

        visit::visit_expr_call(self, node);
    }
}

fn analyze_procedure(
    function: &ItemFn,
    path: &Path,
    project_index: Option<&ProjectAuditIndex>,
) -> Option<ProcedureAnalysis> {
    if !has_procedure_attribute(&function.attrs) {
        return None;
    }

    let mut inputs = procedure_string_inputs(function);

    if inputs.is_empty() {
        return Some(ProcedureAnalysis {
            inputs,
            persistence_sinks: Vec::new(),
        });
    }

    let external_inputs = inputs.iter().map(|input| input.name.clone()).collect();

    let mut flow = ProcedureFlowVisitor {
        path,
        project_index,
        external_inputs,
        from_untrusted_bindings: HashMap::new(),
        protected_bindings: HashMap::new(),
        command_bindings: HashMap::new(),
        delegated_boundaries: HashMap::new(),
        persistence_sinks: Vec::new(),
    };

    flow.visit_block(&function.block);

    for input in &mut inputs {
        input.boundary = flow.boundary_for(&input.name);
    }

    Some(ProcedureAnalysis {
        inputs,
        persistence_sinks: flow.persistence_sinks,
    })
}

fn has_procedure_attribute(attributes: &[Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        attribute
            .path()
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "procedure")
    })
}

fn procedure_string_inputs(function: &ItemFn) -> Vec<ProcedureInput> {
    function
        .sig
        .inputs
        .iter()
        .filter_map(|argument| {
            let FnArg::Typed(argument) = argument else {
                return None;
            };

            if !is_string_type(&argument.ty) {
                return None;
            }

            let Pat::Ident(pattern) = argument.pat.as_ref() else {
                return None;
            };

            Some(ProcedureInput {
                name: pattern.ident.to_string(),
                span: argument.span(),
                boundary: None,
            })
        })
        .collect()
}

fn is_string_type(ty: &Type) -> bool {
    let Type::Path(type_path) = ty else {
        return false;
    };

    if type_path.qself.is_some() {
        return false;
    }

    let Some(segment) = type_path.path.segments.last() else {
        return false;
    };

    segment.ident == "String" && matches!(segment.arguments, PathArguments::None)
}

struct ProcedureFlowVisitor<'a> {
    path: &'a Path,
    project_index: Option<&'a ProjectAuditIndex>,
    external_inputs: HashSet<String>,
    from_untrusted_bindings: HashMap<String, HashSet<String>>,
    protected_bindings: HashMap<String, ProtectedBinding>,
    command_bindings: HashMap<String, HashMap<String, HashSet<String>>>,
    delegated_boundaries: HashMap<String, ProtectionBoundary>,
    persistence_sinks: Vec<PersistenceSinkFlow>,
}

impl ProcedureFlowVisitor<'_> {
    fn boundary_for(&self, input: &str) -> Option<ProtectionBoundary> {
        self.delegated_boundaries.get(input).copied().or_else(|| {
            self.protected_bindings
                .values()
                .find(|binding| binding.sources.contains(input))
                .map(|binding| binding.boundary)
        })
    }

    fn record_persistence_sink(&mut self, call: &ExprCall) {
        let Some((name, specifications)) = persistence_sink_spec(call) else {
            return;
        };

        let arguments = specifications
            .into_iter()
            .map(|(role, index)| {
                let expression = call.args.iter().nth(index);

                self.classify_sink_argument(role, expression)
            })
            .collect();

        self.persistence_sinks.push(PersistenceSinkFlow {
            name,
            span: call.span(),
            arguments,
        });
    }

    fn classify_sink_argument(
        &self,
        role: &'static str,
        expression: Option<&Expr>,
    ) -> PersistenceSinkArgument {
        let binding = expression.and_then(expression_identifier);

        let mut boundary = None;
        let mut sources = Vec::new();

        if let Some(binding_name) = binding.as_deref() {
            if let Some(protected) = self.protected_bindings.get(binding_name) {
                boundary = Some(protected.boundary);
                sources.extend(protected.sources.iter().cloned());
            } else if let Some(untrusted) = self.from_untrusted_bindings.get(binding_name) {
                sources.extend(untrusted.iter().cloned());
            } else if self.external_inputs.contains(binding_name) {
                sources.push(binding_name.to_owned());
            }
        }

        sources.sort();
        sources.dedup();

        PersistenceSinkArgument {
            role,
            binding,
            boundary,
            sources,
        }
    }
}

impl ProcedureFlowVisitor<'_> {
    fn sources_for_expression(&self, expression: &Expr) -> HashSet<String> {
        let Some(binding) = expression_identifier(expression) else {
            return HashSet::new();
        };

        if let Some(protected) = self.protected_bindings.get(&binding) {
            return protected.sources.clone();
        }

        if let Some(untrusted) = self.from_untrusted_bindings.get(&binding) {
            return untrusted.clone();
        }

        if self.external_inputs.contains(&binding) {
            return [binding].into_iter().collect();
        }

        HashSet::new()
    }

    fn record_command_binding(&mut self, binding: &str, expression: &Expr) {
        let Some(project_index) = self.project_index else {
            return;
        };

        let Expr::Call(call) = peel_expr(expression) else {
            return;
        };

        let Expr::Path(function) = call.func.as_ref() else {
            return;
        };

        let segments: Vec<_> = function.path.segments.iter().collect();

        if segments.len() < 2 {
            return;
        }

        let owner = segments[segments.len() - 2].ident.to_string();
        let method = segments[segments.len() - 1].ident.to_string();

        let Some(summary) = project_index.resolve_constructor(&owner, &method).cloned() else {
            return;
        };

        let mut fields = HashMap::new();

        for (field, argument_index) in summary.field_arguments {
            let Some(argument) = call.args.iter().nth(argument_index) else {
                continue;
            };

            let sources = self.sources_for_expression(argument);

            if !sources.is_empty() {
                fields.insert(field, sources);
            }
        }

        if !fields.is_empty() {
            self.command_bindings.insert(binding.to_owned(), fields);
        }
    }

    fn map_symbolic_source(
        &self,
        summary: &FunctionSummary,
        call: &ExprCall,
        source: &SymbolicSource,
    ) -> HashSet<String> {
        let Some(parameter_index) = summary
            .parameters
            .iter()
            .position(|parameter| parameter == &source.root)
        else {
            return HashSet::new();
        };

        let Some(argument) = call.args.iter().nth(parameter_index) else {
            return HashSet::new();
        };

        if source.fields.is_empty() {
            return self.sources_for_expression(argument);
        }

        if source.fields.len() != 1 {
            return HashSet::new();
        }

        let Some(binding) = expression_identifier(argument) else {
            return HashSet::new();
        };

        self.command_bindings
            .get(&binding)
            .and_then(|fields| fields.get(&source.fields[0]))
            .cloned()
            .unwrap_or_default()
    }

    fn record_application_call(&mut self, call: &ExprCall) {
        let Some(project_index) = self.project_index else {
            return;
        };

        let Expr::Path(function) = call.func.as_ref() else {
            return;
        };

        // This first implementation intentionally follows only imported or
        // locally named free functions. Associated/external calls remain
        // outside the trusted interprocedural boundary.
        if function.path.segments.len() != 1 {
            return;
        }

        let local_name = function.path.segments.first().unwrap().ident.to_string();

        let Some(summary) = project_index
            .resolve_function(self.path, &local_name)
            .cloned()
        else {
            return;
        };

        if summary.parameters.len() != call.args.len() {
            return;
        }

        for protection in &summary.protected_sources {
            let sources = self.map_symbolic_source(&summary, call, &protection.source);

            for source in sources {
                if self.external_inputs.contains(&source) {
                    self.delegated_boundaries
                        .entry(source)
                        .or_insert(protection.boundary);
                }
            }
        }

        for sink in &summary.persistence_sinks {
            let arguments = sink
                .arguments
                .iter()
                .map(|argument| {
                    let mut sources = HashSet::new();

                    for source in &argument.sources {
                        sources.extend(self.map_symbolic_source(&summary, call, source));
                    }

                    let mut sources: Vec<_> = sources.into_iter().collect();
                    sources.sort();

                    PersistenceSinkArgument {
                        role: argument.role,
                        binding: argument.binding.clone(),
                        boundary: if sources.is_empty() {
                            None
                        } else {
                            argument.boundary
                        },
                        sources,
                    }
                })
                .collect();

            self.persistence_sinks.push(PersistenceSinkFlow {
                name: sink.name,
                // Report against the procedure call site. The sink itself
                // lives in another source file.
                span: call.span(),
                arguments,
            });
        }
    }
}

impl<'ast> Visit<'ast> for ProcedureFlowVisitor<'_> {
    fn visit_local(&mut self, node: &'ast Local) {
        let Some(binding) = local_binding_name(&node.pat) else {
            visit::visit_local(self, node);
            return;
        };

        if let Some(init) = &node.init {
            self.record_command_binding(&binding, &init.expr);

            if let Some(sources) = from_untrusted_sources(&init.expr, &self.external_inputs) {
                self.from_untrusted_bindings
                    .insert(binding.clone(), sources.into_iter().collect());
            }

            if let Some(source_binding) = validated_binding_source(&init.expr)
                && let Some(sources) = self.from_untrusted_bindings.get(&source_binding).cloned()
            {
                self.protected_bindings.insert(
                    binding.clone(),
                    ProtectedBinding {
                        boundary: ProtectionBoundary::ValidatedDomainInput,
                        sources,
                    },
                );
            }

            if let Some(source) = uuid_parse_external_source(&init.expr, &self.external_inputs) {
                self.protected_bindings.insert(
                    binding,
                    ProtectedBinding {
                        boundary: ProtectionBoundary::UuidParse,
                        sources: [source].into_iter().collect(),
                    },
                );
            }
        }

        visit::visit_local(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        self.record_persistence_sink(node);
        self.record_application_call(node);
        visit::visit_expr_call(self, node);
    }
}

fn collect_use_aliases(file: &syn::File) -> HashMap<String, String> {
    let mut aliases = HashMap::new();

    for item in &file.items {
        let Item::Use(item_use) = item else {
            continue;
        };

        collect_use_tree_aliases(&item_use.tree, &mut aliases);
    }

    aliases
}

fn collect_use_tree_aliases(tree: &UseTree, aliases: &mut HashMap<String, String>) {
    match tree {
        UseTree::Rename(rename) => {
            aliases.insert(rename.rename.to_string(), rename.ident.to_string());
        }

        UseTree::Group(group) => {
            for item in &group.items {
                collect_use_tree_aliases(item, aliases);
            }
        }

        UseTree::Path(path) => {
            collect_use_tree_aliases(&path.tree, aliases);
        }

        UseTree::Name(_) | UseTree::Glob(_) => {}
    }
}

fn impl_owner_name(ty: &Type) -> Option<String> {
    let Type::Path(type_path) = ty else {
        return None;
    };

    if type_path.qself.is_some() {
        return None;
    }

    Some(type_path.path.segments.last()?.ident.to_string())
}

fn summarize_constructor(owner: &str, method: &syn::ImplItemFn) -> Option<ConstructorSummary> {
    if method.sig.ident != "new" {
        return None;
    }

    let parameters = method_parameter_names(method)?;

    let Stmt::Expr(expression, None) = method.block.stmts.last()? else {
        return None;
    };

    let Expr::Struct(structure) = peel_expr(expression) else {
        return None;
    };

    if structure.rest.is_some() {
        return None;
    }

    let constructed = structure.path.segments.last()?.ident.to_string();

    if constructed != "Self" && constructed != owner {
        return None;
    }

    let mut field_arguments = HashMap::new();

    for field in &structure.fields {
        let Member::Named(field_name) = &field.member else {
            return None;
        };

        let source = expression_identifier(&field.expr)?;
        let argument_index = parameters
            .iter()
            .position(|parameter| parameter == &source)?;

        field_arguments.insert(field_name.to_string(), argument_index);
    }

    if field_arguments.is_empty() {
        None
    } else {
        Some(ConstructorSummary { field_arguments })
    }
}

fn method_parameter_names(method: &syn::ImplItemFn) -> Option<Vec<String>> {
    let mut parameters = Vec::new();

    for argument in &method.sig.inputs {
        let FnArg::Typed(argument) = argument else {
            return None;
        };

        let Pat::Ident(pattern) = argument.pat.as_ref() else {
            return None;
        };

        parameters.push(pattern.ident.to_string());
    }

    Some(parameters)
}

fn summarize_function(function: &ItemFn) -> Option<FunctionSummary> {
    let parameters = function_parameter_names(function)?;
    let parameter_set = parameters.iter().cloned().collect();

    let mut flow = FunctionFlowVisitor {
        parameters: parameter_set,
        from_untrusted_bindings: HashMap::new(),
        protected_bindings: HashMap::new(),
        persistence_sinks: Vec::new(),
    };

    flow.visit_block(&function.block);

    let mut protections: HashMap<SymbolicSource, ProtectionBoundary> = HashMap::new();

    for binding in flow.protected_bindings.values() {
        for source in &binding.sources {
            protections
                .entry(source.clone())
                .or_insert(binding.boundary);
        }
    }

    let mut protected_sources: Vec<_> = protections
        .into_iter()
        .map(|(source, boundary)| SymbolicProtection { source, boundary })
        .collect();

    protected_sources.sort_by(|left, right| left.source.cmp(&right.source));

    Some(FunctionSummary {
        parameters,
        protected_sources,
        persistence_sinks: flow.persistence_sinks,
    })
}

fn function_parameter_names(function: &ItemFn) -> Option<Vec<String>> {
    let mut parameters = Vec::new();

    for argument in &function.sig.inputs {
        let FnArg::Typed(argument) = argument else {
            return None;
        };

        let Pat::Ident(pattern) = argument.pat.as_ref() else {
            return None;
        };

        parameters.push(pattern.ident.to_string());
    }

    Some(parameters)
}

struct SymbolicProtectedBinding {
    boundary: ProtectionBoundary,
    sources: HashSet<SymbolicSource>,
}

struct FunctionFlowVisitor {
    parameters: HashSet<String>,
    from_untrusted_bindings: HashMap<String, HashSet<SymbolicSource>>,
    protected_bindings: HashMap<String, SymbolicProtectedBinding>,
    persistence_sinks: Vec<FunctionSinkFlow>,
}

impl FunctionFlowVisitor {
    fn record_persistence_sink(&mut self, call: &ExprCall) {
        let Some((name, specifications)) = persistence_sink_spec(call) else {
            return;
        };

        let arguments = specifications
            .into_iter()
            .map(|(role, index)| {
                let expression = call.args.iter().nth(index);
                self.classify_sink_argument(role, expression)
            })
            .collect();

        self.persistence_sinks
            .push(FunctionSinkFlow { name, arguments });
    }

    fn classify_sink_argument(
        &self,
        role: &'static str,
        expression: Option<&Expr>,
    ) -> FunctionSinkArgument {
        let binding = expression.and_then(expression_identifier);

        let mut boundary = None;
        let mut sources = Vec::new();

        if let Some(binding_name) = binding.as_deref() {
            if let Some(protected) = self.protected_bindings.get(binding_name) {
                boundary = Some(protected.boundary);
                sources.extend(protected.sources.iter().cloned());
            } else if let Some(untrusted) = self.from_untrusted_bindings.get(binding_name) {
                sources.extend(untrusted.iter().cloned());
            }
        }

        if sources.is_empty()
            && let Some(expression) = expression
            && let Some(source) = symbolic_source(expression, &self.parameters)
        {
            sources.push(source);
        }

        sources.sort();
        sources.dedup();

        FunctionSinkArgument {
            role,
            binding,
            boundary,
            sources,
        }
    }
}

impl<'ast> Visit<'ast> for FunctionFlowVisitor {
    fn visit_local(&mut self, node: &'ast Local) {
        let Some(binding) = local_binding_name(&node.pat) else {
            visit::visit_local(self, node);
            return;
        };

        if let Some(init) = &node.init {
            if let Some(sources) = from_untrusted_symbolic_sources(&init.expr, &self.parameters) {
                self.from_untrusted_bindings
                    .insert(binding.clone(), sources.into_iter().collect());
            }

            if let Some(source_binding) = validated_binding_source(&init.expr)
                && let Some(sources) = self.from_untrusted_bindings.get(&source_binding).cloned()
            {
                self.protected_bindings.insert(
                    binding,
                    SymbolicProtectedBinding {
                        boundary: ProtectionBoundary::ValidatedDomainInput,
                        sources,
                    },
                );
            }
        }

        visit::visit_local(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        self.record_persistence_sink(node);
        visit::visit_expr_call(self, node);
    }
}

fn from_untrusted_symbolic_sources(
    expression: &Expr,
    parameters: &HashSet<String>,
) -> Option<Vec<SymbolicSource>> {
    let Expr::Call(call) = peel_expr(expression) else {
        return None;
    };

    let Expr::Path(function) = call.func.as_ref() else {
        return None;
    };

    let last = function.path.segments.last()?;

    if last.ident != "from_untrusted" {
        return None;
    }

    Some(
        call.args
            .iter()
            .filter_map(|argument| symbolic_source(argument, parameters))
            .collect(),
    )
}

fn symbolic_source(expression: &Expr, parameters: &HashSet<String>) -> Option<SymbolicSource> {
    match peel_expr(expression) {
        Expr::Path(path) => {
            if path.qself.is_some() || path.path.segments.len() != 1 {
                return None;
            }

            let root = path.path.segments.first()?.ident.to_string();

            if !parameters.contains(&root) {
                return None;
            }

            Some(SymbolicSource {
                root,
                fields: Vec::new(),
            })
        }

        Expr::Field(field) => {
            let mut source = symbolic_source(&field.base, parameters)?;

            let Member::Named(member) = &field.member else {
                return None;
            };

            source.fields.push(member.to_string());
            Some(source)
        }

        _ => None,
    }
}

fn local_binding_name(pattern: &Pat) -> Option<String> {
    let Pat::Ident(pattern) = pattern else {
        return None;
    };

    Some(pattern.ident.to_string())
}

fn from_untrusted_sources(
    expression: &Expr,
    external_inputs: &HashSet<String>,
) -> Option<Vec<String>> {
    let Expr::Call(call) = peel_expr(expression) else {
        return None;
    };

    let Expr::Path(function) = call.func.as_ref() else {
        return None;
    };

    let last = function.path.segments.last()?;

    if last.ident != "from_untrusted" {
        return None;
    }

    Some(
        call.args
            .iter()
            .filter_map(expression_identifier)
            .filter(|name| external_inputs.contains(name))
            .collect(),
    )
}

fn validated_binding_source(expression: &Expr) -> Option<String> {
    match peel_expr(expression) {
        Expr::Match(expression) => {
            let source = validate_receiver_identifier(&expression.expr)?;

            if match_returns_ok_value(expression) {
                Some(source)
            } else {
                None
            }
        }

        Expr::MethodCall(expression)
            if expression.method == "expect" || expression.method == "unwrap" =>
        {
            validate_receiver_identifier(&expression.receiver)
        }

        Expr::Try(expression) => validate_receiver_identifier(&expression.expr),

        _ => None,
    }
}

fn validate_receiver_identifier(expression: &Expr) -> Option<String> {
    let Expr::MethodCall(method) = peel_expr(expression) else {
        return None;
    };

    if method.method != "validate" || !method.args.is_empty() {
        return None;
    }

    expression_identifier(&method.receiver)
}

fn match_returns_ok_value(expression: &syn::ExprMatch) -> bool {
    expression.arms.iter().any(|arm| {
        let Pat::TupleStruct(pattern) = &arm.pat else {
            return false;
        };

        let Some(last) = pattern.path.segments.last() else {
            return false;
        };

        if last.ident != "Ok" || pattern.elems.len() != 1 {
            return false;
        }

        let Some(Pat::Ident(value)) = pattern.elems.first() else {
            return false;
        };

        expression_identifier(&arm.body).is_some_and(|body| value.ident == body)
    })
}

fn expression_identifier(expression: &Expr) -> Option<String> {
    let Expr::Path(path) = peel_expr(expression) else {
        return None;
    };

    if path.qself.is_some() || path.path.segments.len() != 1 {
        return None;
    }

    Some(path.path.segments.first()?.ident.to_string())
}

fn uuid_parse_external_source(
    expression: &Expr,
    external_inputs: &HashSet<String>,
) -> Option<String> {
    let mut visitor = UuidParseVisitor {
        external_inputs,
        source: None,
    };

    visitor.visit_expr(expression);

    visitor.source
}

struct UuidParseVisitor<'a> {
    external_inputs: &'a HashSet<String>,
    source: Option<String>,
}

impl<'ast> Visit<'ast> for UuidParseVisitor<'_> {
    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        if self.source.is_none()
            && let Some(source) = uuid_parse_source(node)
            && self.external_inputs.contains(&source)
        {
            self.source = Some(source);
            return;
        }

        visit::visit_expr_call(self, node);
    }
}

fn uuid_parse_source(call: &ExprCall) -> Option<String> {
    let Expr::Path(function) = call.func.as_ref() else {
        return None;
    };

    let segments: Vec<_> = function.path.segments.iter().collect();

    if segments.len() < 2 {
        return None;
    }

    let owner = &segments[segments.len() - 2].ident;
    let method = &segments[segments.len() - 1].ident;

    if owner != "Uuid" || method != "parse_str" {
        return None;
    }

    expression_identifier(call.args.first()?)
}

fn persistence_sink_spec(call: &ExprCall) -> Option<(&'static str, Vec<(&'static str, usize)>)> {
    let Expr::Path(function) = call.func.as_ref() else {
        return None;
    };

    let segments: Vec<_> = function.path.segments.iter().collect();

    if segments.len() < 2 {
        return None;
    }

    let owner = &segments[segments.len() - 2].ident;
    let method = &segments[segments.len() - 1].ident;

    if owner != "Contact" {
        return None;
    }

    match method.to_string().as_str() {
        "create" => Some(("Contact::create", vec![("input", 1)])),

        "update" => Some(("Contact::update", vec![("id", 1), ("input", 2)])),

        "delete" => Some(("Contact::delete", vec![("id", 1)])),

        _ => None,
    }
}

fn format_sink_argument(argument: &PersistenceSinkArgument) -> String {
    let binding = argument.binding.as_deref().unwrap_or("<expression>");

    let boundary = argument
        .boundary
        .map(ProtectionBoundary::label)
        .unwrap_or("unrecognized");

    if argument.sources.is_empty() {
        return format!("{} `{binding}` -> {boundary}", argument.role);
    }

    format!(
        "{} `{binding}` <- [{}] -> {boundary}",
        argument.role,
        argument.sources.join(", "),
    )
}

fn shell_interpreter(call: &ExprCall) -> Option<String> {
    let Expr::Path(function) = call.func.as_ref() else {
        return None;
    };

    let segments: Vec<_> = function.path.segments.iter().collect();

    if segments.len() < 2 {
        return None;
    }

    let command = segments[segments.len() - 2].ident.to_string();
    let method = segments[segments.len() - 1].ident.to_string();

    if command != "Command" || method != "new" {
        return None;
    }

    let argument = call.args.first()?;

    let Expr::Lit(literal) = peel_expr(argument) else {
        return None;
    };

    let Lit::Str(value) = &literal.lit else {
        return None;
    };

    let executable = value.value();

    if is_shell_interpreter(&executable) {
        Some(executable)
    } else {
        None
    }
}

fn is_shell_interpreter(executable: &str) -> bool {
    let executable = executable.to_ascii_lowercase();

    let basename = executable.rsplit(['/', '\\']).next().unwrap_or(&executable);

    matches!(
        basename,
        "sh" | "bash"
            | "dash"
            | "zsh"
            | "cmd"
            | "cmd.exe"
            | "powershell"
            | "powershell.exe"
            | "pwsh"
            | "pwsh.exe"
    )
}

fn dynamic_sql_function(call: &ExprCall) -> Option<String> {
    let Expr::Path(function) = call.func.as_ref() else {
        return None;
    };

    let mut segments = function.path.segments.iter();

    let first = segments.next()?;

    if first.ident != "sqlx" {
        return None;
    }

    let last = function.path.segments.last()?;

    let function_name = last.ident.to_string();

    let recognized = matches!(
        function_name.as_str(),
        "query"
            | "query_as"
            | "query_scalar"
            | "query_with"
            | "query_as_with"
            | "query_scalar_with"
            | "raw_sql"
    );

    if !recognized {
        return None;
    }

    let first_argument = call.args.first()?;

    if is_format_macro(first_argument) {
        Some(format!("sqlx::{function_name}"))
    } else {
        None
    }
}

fn is_format_macro(expression: &Expr) -> bool {
    let expression = peel_expr(expression);

    let Expr::Macro(expression) = expression else {
        return false;
    };

    expression
        .mac
        .path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "format")
}

fn peel_expr(mut expression: &Expr) -> &Expr {
    loop {
        expression = match expression {
            Expr::Reference(reference) => &reference.expr,
            Expr::Paren(paren) => &paren.expr,
            Expr::Group(group) => &group.expr,
            _ => return expression,
        };
    }
}

#[cfg(test)]
mod tests;
