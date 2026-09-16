//! # Microsoft Access 365 Platform Integration Subsystem for Tagisan
//!
//! Provides enterprise-grade Microsoft Access 365, Jet/ACE (Access Database Engine),
//! and DAO/VBA integration:
//!
//! 1. `AceSqlTranspiler`:
//!    - Transpiles ANSI SQL to strict Jet/ACE SQL.
//!    - Mandatory nested parenthesized binary joins:
//!      `((A INNER JOIN B ON ...) LEFT JOIN C ON ...)`
//!    - Function mapping: `COALESCE` -> `Nz()`, `CASE WHEN` -> nested `IIf()`,
//!      string concatenation `||` -> `&`, `SUBSTRING` -> `Mid`, `LENGTH` -> `Len`.
//!    - Literal conversions: ANSI date/timestamp strings -> `#yyyy-mm-dd#` or `#yyyy-mm-dd hh:nn:ss#`.
//!    - Cartesian product detection: flags queries lacking join predicates (O(N*M) hazards).
//!    - Syntax error detection & invariant checking.
//!
//! 2. `AccessSchemaEngine`:
//!    - Defines tables, columns, indexes, and foreign keys.
//!    - Full Access data type mapping (ShortText, LongText, Byte, Integer, LongInteger,
//!      SingleFloat, DoubleFloat, Currency, DateTime, AutoNumber, YesNo, OleObject, Hyperlink, Guid, Decimal).
//!    - Generates strict ACE DDL scripts (`CREATE TABLE`, `CREATE INDEX`, `ALTER TABLE ... ADD CONSTRAINT`).
//!    - Enforces referential integrity with ON UPDATE / ON DELETE CASCADE / SET NULL rules.
//!    - Validates schema topology (missing PKs, invalid FK targets, circular dependencies).
//!
//! 3. `AccessFormReportGenerator`:
//!    - Plain text generation compliant with Microsoft Access `SaveAsText` / `LoadFromText`.
//!    - Twip coordinate geometry: exactly 1440 twips = 1 inch (567 twips = 1 cm, 20 twips = 1 pt).
//!    - Forms: TextBox, ComboBox, CommandButton, CheckBox, Label, Subform controls with properties.
//!    - Banded Reports: ReportHeader, PageHeader, GroupHeader, Detail, GroupFooter, PageFooter,
//!      ReportFooter with running sums (`RunningSum = 1` for Over All, `RunningSum = 2` for Over Group).
//!
//! 4. `AccessVbaBridge`:
//!    - Production-grade 64-bit `PtrSafe` VBA modules with conditional compilation (`#If VBA7 Then ... #Else ... #End If`).
//!    - Win32/Win64 C-ABI interop declarations (`LongPtr` handle safety).
//!    - WinHTTP REST client targeting `http://localhost:3978/api/copilot/access` with JSON payloads,
//!      timeouts, custom headers (`X-Tagisan-Purview-Label`), and robust error trapping.
//!    - DAO workspace transaction wrappers (`ws.BeginTrans`, `ws.CommitTrans dbForceOSFlush`, `ws.Rollback`).
//!
//! 5. `AccessDataverseMigrator`:
//!    - Migrates desktop Access tables to cloud Microsoft Dataverse entities.
//!    - Generates Dataverse Entity Definition JSON and 1:N Relationship Metadata.
//!    - Generates Access linked table scripts using ODBC Driver 18 for SQL Server / Dataverse Connector.
//!    - Field-by-field migration mapping and validation report.
//!
//! 6. `AccessBlastRadiusAnalyzer`:
//!    - Computes blast radius of table or column renames across:
//!      * Saved queries (SELECT, JOIN, WHERE, GROUP BY, ORDER BY).
//!      * Forms (RecordSource, ControlSource, RowSource).
//!      * Reports (RecordSource, ControlSource, running calculations).
//!      * VBA modules (SQL literals, `CurrentDb.OpenRecordset`, `rs!Field`, `Forms!frm!ctl`).
//!    - Emits structured report with risk rating, impacted symbols, and surgical autofix patch.
//!
//! 7. Purview & AgentShield DLP Guard:
//!    - Data exfiltration defense on queries and table schemas.
//!    - Zero-Egress Air-Gapping on Confidential/Secret Access databases.
//!    - Cryptographic SHA-256 `PurviewAuditReceipt` generation.
//!
//! 8. `CopilotAccessTool`:
//!    - Implements `ToolHandler` exposing all capabilities to autonomous agents and MCP clients.

use crate::copilot::purview::{PurviewGuardEngine, PurviewGuardResult, PurviewSensitivity, PurviewAuditReceipt};
use crate::ecc::agentshield::{AgentShieldScanner, AgentShieldVerdict, ThreatLevel};
use crate::error::{Result, TagisanError};
use crate::tools::ToolHandler;
use async_trait::async_trait;
use chrono::Utc;
use regex::{Regex, RegexBuilder};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tracing::{debug, info, warn};

// =========================================================================
// SECTION 1: AceSqlTranspiler (ANSI SQL -> Strict Jet/ACE Dialect)
// =========================================================================

/// Transpilation result with diagnostics, telemetry, and Cartesian warnings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TranspileResult {
    pub original_sql: String,
    pub transpiled_sql: String,
    pub tables_referenced: Vec<String>,
    pub columns_referenced: Vec<String>,
    pub is_cartesian: bool,
    pub join_depth: usize,
    pub warnings: Vec<String>,
    pub purview_sensitivity: PurviewSensitivity,
    pub audit_receipt: Option<PurviewAuditReceipt>,
}

/// Transpiles ANSI SQL to strict Jet/ACE SQL dialect
#[derive(Debug, Clone, Default)]
pub struct AceSqlTranspiler;

impl AceSqlTranspiler {
    pub fn new() -> Self {
        Self
    }

    /// Primary entry point: transpiles ANSI SQL into Jet/ACE SQL
    pub fn transpile(&self, sql: &str) -> Result<TranspileResult> {
        let trimmed = sql.trim();
        if trimmed.is_empty() {
            return Err(TagisanError::Execution("SQL query cannot be empty".to_string()));
        }

        // 1. Basic Invariant & Balanced Parentheses Check
        self.verify_syntax_balance(trimmed)?;

        let mut warnings = Vec::new();

        // 2. Transpile CASE WHEN -> nested IIf()
        let case_transpiled = self.transpile_case_when(trimmed)?;

        // 3. Transpile COALESCE -> nested Nz()
        let coalesce_transpiled = self.transpile_coalesce(&case_transpiled)?;

        // 4. Transpile ANSI Date Literals to #yyyy-mm-dd# or #yyyy-mm-dd hh:nn:ss#
        let date_transpiled = self.transpile_date_literals(&coalesce_transpiled);

        // 5. Transpile string concatenation (|| -> &) and common scalar functions
        let functions_transpiled = self.transpile_functions_and_operators(&date_transpiled);

        // 6. Transpile JOIN clauses into strict parenthesized binary joins
        let (joins_transpiled, join_depth, tables_detected) = self.transpile_joins(&functions_transpiled)?;

        // 7. Cartesian Product Detection
        let is_cartesian = self.detect_cartesian_product(&joins_transpiled, &tables_detected, &mut warnings);

        // 8. Extract Column References (Best-effort heuristic AST extraction)
        let columns_detected = self.extract_column_references(&joins_transpiled);

        // 9. Purview Sensitivity Classification
        let sensitivity = PurviewGuardEngine::classify(&joins_transpiled, None);
        let audit_receipt = if sensitivity.is_air_gapped() {
            let eval = PurviewGuardEngine::evaluate(&joins_transpiled, Some(sensitivity.as_str()), None)?;
            Some(eval.receipt)
        } else {
            None
        };

        Ok(TranspileResult {
            original_sql: sql.to_string(),
            transpiled_sql: joins_transpiled,
            tables_referenced: tables_detected,
            columns_referenced: columns_detected,
            is_cartesian,
            join_depth,
            warnings,
            purview_sensitivity: sensitivity,
            audit_receipt,
        })
    }

    /// Verifies balanced parentheses, quotes, and brackets
    fn verify_syntax_balance(&self, sql: &str) -> Result<()> {
        let mut paren_count = 0isize;
        let mut bracket_count = 0isize;
        let mut in_single_quote = false;
        let mut prev_char = ' ';

        for c in sql.chars() {
            if c == '\'' && prev_char != '\\' {
                in_single_quote = !in_single_quote;
            } else if !in_single_quote {
                match c {
                    '(' => paren_count += 1,
                    ')' => {
                        paren_count -= 1;
                        if paren_count < 0 {
                            return Err(TagisanError::Execution(
                                "Syntax error: Unexpected closing parenthesis ')' without matching opening".to_string(),
                            ));
                        }
                    }
                    '[' => bracket_count += 1,
                    ']' => {
                        bracket_count -= 1;
                        if bracket_count < 0 {
                            return Err(TagisanError::Execution(
                                "Syntax error: Unexpected closing bracket ']' without matching opening".to_string(),
                            ));
                        }
                    }
                    _ => {}
                }
            }
            prev_char = c;
        }

        if in_single_quote {
            return Err(TagisanError::Execution("Syntax error: Unterminated string literal".to_string()));
        }
        if paren_count != 0 {
            return Err(TagisanError::Execution(format!(
                "Syntax error: Unbalanced parentheses ({} unclosed opening parentheses)",
                paren_count
            )));
        }
        if bracket_count != 0 {
            return Err(TagisanError::Execution(format!(
                "Syntax error: Unbalanced square brackets ({} unclosed opening brackets)",
                bracket_count
            )));
        }

        Ok(())
    }

    /// Transpiles CASE WHEN ... THEN ... ELSE ... END into nested IIf(cond, then, else)
    pub fn transpile_case_when(&self, sql: &str) -> Result<String> {
        let case_regex = RegexBuilder::new(r"(?i)\bCASE\b([\s\S]*?)\bEND\b")
            .case_insensitive(true)
            .build()
            .map_err(|e| TagisanError::Execution(e.to_string()))?;

        let mut output = String::with_capacity(sql.len());
        let mut last_end = 0;

        for cap in case_regex.captures_iter(sql) {
            let m = cap.get(0).unwrap();
            output.push_str(&sql[last_end..m.start()]);

            let inner = cap.get(1).unwrap().as_str();
            let converted = self.convert_single_case(inner)?;
            output.push_str(&converted);

            last_end = m.end();
        }
        output.push_str(&sql[last_end..]);

        Ok(output)
    }

    fn convert_single_case(&self, inner: &str) -> Result<String> {
        // Look for WHEN ... THEN ... and optional ELSE ...
        let when_then_regex = RegexBuilder::new(r"(?i)\bWHEN\b([\s\S]*?)\bTHEN\b")
            .case_insensitive(true)
            .build()
            .map_err(|e| TagisanError::Execution(e.to_string()))?;

        let else_regex = RegexBuilder::new(r"(?i)\bELSE\b([\s\S]*)$")
            .case_insensitive(true)
            .build()
            .map_err(|e| TagisanError::Execution(e.to_string()))?;

        // Extract ELSE if present
        let (body, else_clause) = if let Some(else_cap) = else_regex.captures(inner) {
            let else_val = else_cap.get(1).unwrap().as_str().trim().to_string();
            let body_part = &inner[..else_cap.get(0).unwrap().start()];
            (body_part, else_val)
        } else {
            (inner, "Null".to_string())
        };

        // Collect all WHEN ... THEN pairs
        let mut pairs = Vec::new();
        let mut last_end = 0;
        let mut current_cond = None;

        for cap in when_then_regex.captures_iter(body) {
            let m = cap.get(0).unwrap();
            if let Some(cond) = current_cond.take() {
                let then_val = body[last_end..m.start()].trim().to_string();
                pairs.push((cond, then_val));
            }
            current_cond = Some(cap.get(1).unwrap().as_str().trim().to_string());
            last_end = m.end();
        }

        if let Some(cond) = current_cond {
            let then_val = body[last_end..].trim().to_string();
            pairs.push((cond, then_val));
        }

        if pairs.is_empty() {
            return Err(TagisanError::Execution(format!(
                "Malformed CASE statement lacking WHEN clauses: '{}'",
                inner
            )));
        }

        // Fold rightwards: IIf(cond_1, val_1, IIf(cond_2, val_2, else_val))
        let mut expr = else_clause;
        for (cond, val) in pairs.into_iter().rev() {
            expr = format!("IIf({}, {}, {})", cond, val, expr);
        }

        Ok(expr)
    }

    /// Transpiles COALESCE(arg1, arg2, ...) into nested Nz(arg1, Nz(arg2, ...))
    pub fn transpile_coalesce(&self, sql: &str) -> Result<String> {
        let coalesce_pattern = RegexBuilder::new(r"(?i)\bCOALESCE\s*\(")
            .case_insensitive(true)
            .build()
            .map_err(|e| TagisanError::Execution(e.to_string()))?;

        let mut output = String::with_capacity(sql.len());
        let mut cur_idx = 0;

        while let Some(m) = coalesce_pattern.find_at(sql, cur_idx) {
            output.push_str(&sql[cur_idx..m.start()]);

            // Find matching closing parenthesis
            let open_paren_pos = m.end() - 1;
            let mut depth = 1;
            let mut end_pos = None;
            let mut in_quote = false;

            for (i, c) in sql[open_paren_pos + 1..].char_indices() {
                let abs_pos = open_paren_pos + 1 + i;
                if c == '\'' {
                    in_quote = !in_quote;
                } else if !in_quote {
                    if c == '(' {
                        depth += 1;
                    } else if c == ')' {
                        depth -= 1;
                        if depth == 0 {
                            end_pos = Some(abs_pos);
                            break;
                        }
                    }
                }
            }

            let close_pos = end_pos.ok_or_else(|| {
                TagisanError::Execution("Unclosed parenthesis in COALESCE function".to_string())
            })?;

            let args_str = &sql[open_paren_pos + 1..close_pos];
            let converted = self.convert_coalesce_args(args_str);
            output.push_str(&converted);

            cur_idx = close_pos + 1;
        }

        output.push_str(&sql[cur_idx..]);
        Ok(output)
    }

    fn convert_coalesce_args(&self, args_str: &str) -> String {
        let args = self.split_top_level_commas(args_str);
        if args.is_empty() {
            return "Nz()".to_string();
        }
        if args.len() == 1 {
            return format!("Nz({}, \"\")", args[0].trim());
        }
        if args.len() == 2 {
            return format!("Nz({}, {})", args[0].trim(), args[1].trim());
        }

        // Multi-argument coalesce: Nz(a, Nz(b, c))
        let mut acc = args.last().unwrap().trim().to_string();
        for arg in args[..args.len() - 1].iter().rev() {
            acc = format!("Nz({}, {})", arg.trim(), acc);
        }
        acc
    }

    fn split_top_level_commas(&self, s: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut cur = String::new();
        let mut depth = 0;
        let mut in_quote = false;

        for c in s.chars() {
            if c == '\'' {
                in_quote = !in_quote;
                cur.push(c);
            } else if in_quote {
                cur.push(c);
            } else if c == '(' || c == '[' {
                depth += 1;
                cur.push(c);
            } else if c == ')' || c == ']' {
                depth -= 1;
                cur.push(c);
            } else if c == ',' && depth == 0 {
                tokens.push(cur.trim().to_string());
                cur.clear();
            } else {
                cur.push(c);
            }
        }
        if !cur.trim().is_empty() {
            tokens.push(cur.trim().to_string());
        }
        tokens
    }

    /// Transpiles ANSI date literals to Access #yyyy-mm-dd# syntax
    pub fn transpile_date_literals(&self, sql: &str) -> String {
        let re_date_keyword = RegexBuilder::new(r"(?i)\bDATE\s*'(\d{4}-\d{2}-\d{2})'")
            .case_insensitive(true)
            .build()
            .unwrap();
        let step1 = re_date_keyword.replace_all(sql, "#$1#").to_string();

        let re_timestamp = RegexBuilder::new(r"'(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2})'")
            .build()
            .unwrap();
        let step2 = re_timestamp.replace_all(&step1, "#$1#").to_string();

        let re_iso_date = RegexBuilder::new(r"(?i)(=\s*|<>\s*|<=\s*|>=\s*|<\s*|>\s*|\bBETWEEN\s+|\bAND\s+)'(\d{4}-\d{2}-\d{2})'")
            .case_insensitive(true)
            .build()
            .unwrap();
        re_iso_date.replace_all(&step2, "$1#$2#").to_string()
    }

    /// Transpiles ANSI functions and operators: || -> &, SUBSTRING -> Mid, LENGTH -> Len, etc.
    pub fn transpile_functions_and_operators(&self, sql: &str) -> String {
        let mut res = sql.to_string();

        let re_concat = Regex::new(r"\|\|").unwrap();
        res = re_concat.replace_all(&res, "&").to_string();

        let re_substr_from = RegexBuilder::new(r"(?i)\bSUBSTRING\s*\(\s*([^,\(\)]+)\s+FROM\s+(\d+)\s+FOR\s+(\d+)\s*\)")
            .case_insensitive(true)
            .build()
            .unwrap();
        res = re_substr_from.replace_all(&res, "Mid($1, $2, $3)").to_string();

        let re_substr = RegexBuilder::new(r"(?i)\bSUBSTRING\s*\(")
            .case_insensitive(true)
            .build()
            .unwrap();
        res = re_substr.replace_all(&res, "Mid(").to_string();

        let re_substr_short = RegexBuilder::new(r"(?i)\bSUBSTR\s*\(")
            .case_insensitive(true)
            .build()
            .unwrap();
        res = re_substr_short.replace_all(&res, "Mid(").to_string();

        let re_len = RegexBuilder::new(r"(?i)\bLENGTH\s*\(")
            .case_insensitive(true)
            .build()
            .unwrap();
        res = re_len.replace_all(&res, "Len(").to_string();

        let re_cur_ts = RegexBuilder::new(r"(?i)\bCURRENT_TIMESTAMP\b")
            .case_insensitive(true)
            .build()
            .unwrap();
        res = re_cur_ts.replace_all(&res, "Now()").to_string();

        let re_now = RegexBuilder::new(r"(?i)\bNOW\s*\(\s*\)")
            .case_insensitive(true)
            .build()
            .unwrap();
        res = re_now.replace_all(&res, "Now()").to_string();

        let re_cur_date = RegexBuilder::new(r"(?i)\bCURRENT_DATE\b")
            .case_insensitive(true)
            .build()
            .unwrap();
        res = re_cur_date.replace_all(&res, "Date()").to_string();

        let re_ifnull = RegexBuilder::new(r"(?i)\b(IFNULL|ISNULL)\s*\(")
            .case_insensitive(true)
            .build()
            .unwrap();
        res = re_ifnull.replace_all(&res, "Nz(").to_string();

        let re_true = RegexBuilder::new(r"(?i)\bTRUE\b")
            .case_insensitive(true)
            .build()
            .unwrap();
        res = re_true.replace_all(&res, "True").to_string();

        let re_false = RegexBuilder::new(r"(?i)\bFALSE\b")
            .case_insensitive(true)
            .build()
            .unwrap();
        res = re_false.replace_all(&res, "False").to_string();

        res
    }

    /// Transpiles JOIN clauses into strict parenthesized binary joins
    pub fn transpile_joins(&self, sql: &str) -> Result<(String, usize, Vec<String>)> {
        let from_re = RegexBuilder::new(r"(?i)\bFROM\b")
            .case_insensitive(true)
            .build()
            .map_err(|e| TagisanError::Execution(e.to_string()))?;

        let from_match = match from_re.find(sql) {
            Some(m) => m,
            None => {
                return Ok((sql.to_string(), 0, Vec::new()));
            }
        };

        let from_start = from_match.end();

        let boundary_re = RegexBuilder::new(r"(?i)\b(WHERE|GROUP\s+BY|HAVING|ORDER\s+BY)\b")
            .case_insensitive(true)
            .build()
            .map_err(|e| TagisanError::Execution(e.to_string()))?;

        let (from_clause, tail_part) = if let Some(bm) = boundary_re.find_at(sql, from_start) {
            (&sql[from_start..bm.start()], &sql[bm.start()..])
        } else {
            (&sql[from_start..], "")
        };

        let join_keyword_re = RegexBuilder::new(r"(?i)\b(CROSS\s+JOIN|INNER\s+JOIN|LEFT\s+OUTER\s+JOIN|LEFT\s+JOIN|RIGHT\s+OUTER\s+JOIN|RIGHT\s+JOIN|FULL\s+OUTER\s+JOIN|FULL\s+JOIN|JOIN)\b")
            .case_insensitive(true)
            .build()
            .map_err(|e| TagisanError::Execution(e.to_string()))?;

        let join_matches: Vec<_> = join_keyword_re.find_iter(from_clause).collect();
        if join_matches.is_empty() {
            let tables = self.extract_table_names(from_clause);
            return Ok((sql.to_string(), 0, tables));
        }

        let first_join_start = join_matches[0].start();
        let base_table = from_clause[..first_join_start].trim().to_string();
        let mut tables = vec![self.clean_table_identifier(&base_table)];

        let mut parsed_joins = Vec::new();

        for i in 0..join_matches.len() {
            let join_match = &join_matches[i];
            let join_type_raw = join_match.as_str().to_uppercase();
            let is_cross = join_type_raw.contains("CROSS");
            let join_type = if is_cross {
                ","
            } else if join_type_raw.contains("LEFT") {
                "LEFT JOIN"
            } else if join_type_raw.contains("RIGHT") {
                "RIGHT JOIN"
            } else {
                "INNER JOIN"
            };

            let block_start = join_match.end();
            let block_end = if i + 1 < join_matches.len() {
                join_matches[i + 1].start()
            } else {
                from_clause.len()
            };

            let block_content = &from_clause[block_start..block_end];

            if is_cross {
                let table_name = block_content.trim().to_string();
                tables.push(self.clean_table_identifier(&table_name));
                parsed_joins.push((join_type.to_string(), table_name, String::new()));
                continue;
            }

            let on_re = RegexBuilder::new(r"(?i)\bON\b")
                .case_insensitive(true)
                .build()
                .map_err(|e| TagisanError::Execution(e.to_string()))?;

            let on_match = on_re.find(block_content).ok_or_else(|| {
                TagisanError::Execution(format!(
                    "Syntax error: JOIN missing mandatory ON condition in '{}'",
                    block_content.trim()
                ))
            })?;

            let table_name = block_content[..on_match.start()].trim().to_string();
            let on_condition = block_content[on_match.end()..].trim().to_string();

            tables.push(self.clean_table_identifier(&table_name));
            parsed_joins.push((join_type.to_string(), table_name, on_condition));
        }

        let join_depth = parsed_joins.len();

        let mut accumulated = base_table;
        for (jtype, tbl, cond) in parsed_joins {
            if jtype == "," {
                accumulated = format!("{}, {}", accumulated, tbl);
            } else {
                accumulated = format!("({} {} {} ON {})", accumulated, jtype, tbl, cond);
            }
        }

        let prefix = &sql[..from_start];
        let new_sql = format!("{} {} {}", prefix.trim_end(), accumulated, tail_part.trim_start());

        Ok((new_sql, join_depth, tables))
    }

    fn clean_table_identifier(&self, raw: &str) -> String {
        let parts: Vec<&str> = raw.split_whitespace().collect();
        let name = parts.first().copied().unwrap_or(raw);
        name.trim_matches('[').trim_matches(']').to_string()
    }

    fn extract_table_names(&self, from_clause: &str) -> Vec<String> {
        from_clause
            .split(',')
            .map(|part| self.clean_table_identifier(part.trim()))
            .filter(|s| !s.is_empty())
            .collect()
    }

    fn detect_cartesian_product(
        &self,
        sql: &str,
        tables: &[String],
        warnings: &mut Vec<String>,
    ) -> bool {
        let upper = sql.to_uppercase();

        if upper.contains("CROSS JOIN") {
            warnings.push("Cartesian product detected: explicit CROSS JOIN encountered, causing O(N*M) row explosion".to_string());
            return true;
        }

        if tables.len() >= 2 && !upper.contains("JOIN") {
            if !upper.contains("WHERE") {
                warnings.push(format!(
                    "Cartesian product detected: multiple tables ({}) queried without JOIN predicates or WHERE filter",
                    tables.join(", ")
                ));
                return true;
            } else {
                let where_idx = upper.find("WHERE").unwrap_or(0);
                let where_clause = &upper[where_idx..];
                if !where_clause.contains('=') {
                    warnings.push(format!(
                        "Potential Cartesian product: WHERE clause lacks equality join condition between tables ({})",
                        tables.join(", ")
                    ));
                    return true;
                }
            }
        }

        false
    }

    fn extract_column_references(&self, sql: &str) -> Vec<String> {
        let re_col = Regex::new(r"\[([A-Za-z0-9_]+)\]\.\[([A-Za-z0-9_]+)\]|([A-Za-z0-9_]+)\.([A-Za-z0-9_]+)").unwrap();
        let mut cols = HashSet::new();

        for cap in re_col.captures_iter(sql) {
            if let (Some(tbl), Some(col)) = (cap.get(1), cap.get(2)) {
                cols.insert(format!("{}.{}", tbl.as_str(), col.as_str()));
            } else if let (Some(tbl), Some(col)) = (cap.get(3), cap.get(4)) {
                cols.insert(format!("{}.{}", tbl.as_str(), col.as_str()));
            }
        }

        let mut sorted: Vec<_> = cols.into_iter().collect();
        sorted.sort();
        sorted
    }
}

// =========================================================================
// SECTION 2: AccessSchemaEngine (Data Types, DDL, Referential Integrity)
// =========================================================================

/// Microsoft Access Jet/ACE native data types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessDataType {
    ShortText { max_length: u16 }, // 1..=255
    LongText,                      // MEMO
    Byte,                          // 0..=255
    Integer,                       // 16-bit signed
    LongInteger,                   // 32-bit signed
    SingleFloat,                   // 32-bit IEEE float
    DoubleFloat,                   // 64-bit IEEE float
    Currency,                      // 4 decimal places fixed point
    AutoNumber,                    // Counter identity
    DateTime,                      // Standard Access datetime
    YesNo,                         // Boolean
    OleObject,                     // Binary / Attachment
    Hyperlink,                     // Hyperlink memo
    Guid,                          // Replication ID
    Decimal { precision: u8, scale: u8 },
}

impl AccessDataType {
    pub fn to_ace_ddl(&self) -> String {
        match self {
            Self::ShortText { max_length } => format!("TEXT({})", max_length),
            Self::LongText => "MEMO".to_string(),
            Self::Byte => "BYTE".to_string(),
            Self::Integer => "SHORT".to_string(),
            Self::LongInteger => "LONG".to_string(),
            Self::SingleFloat => "SINGLE".to_string(),
            Self::DoubleFloat => "DOUBLE".to_string(),
            Self::Currency => "CURRENCY".to_string(),
            Self::AutoNumber => "AUTOINCREMENT".to_string(),
            Self::DateTime => "DATETIME".to_string(),
            Self::YesNo => "YESNO".to_string(),
            Self::OleObject => "LONGBINARY".to_string(),
            Self::Hyperlink => "MEMO".to_string(),
            Self::Guid => "GUID".to_string(),
            Self::Decimal { precision, scale } => format!("DECIMAL({}, {})", precision, scale),
        }
    }
}

/// Foreign key referential integrity rule
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReferentialRule {
    Cascade,
    SetNull,
    NoAction,
}

impl ReferentialRule {
    pub fn to_ddl(&self) -> &'static str {
        match self {
            Self::Cascade => "CASCADE",
            Self::SetNull => "SET NULL",
            Self::NoAction => "NO ACTION",
        }
    }
}

/// Column definition in Access table
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: AccessDataType,
    pub is_primary_key: bool,
    pub is_nullable: bool,
    pub is_unique: bool,
    pub default_value: Option<String>,
    pub validation_rule: Option<String>,
    pub validation_text: Option<String>,
    pub description: Option<String>,
}

/// Foreign key constraint
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForeignKeyDefinition {
    pub name: String,
    pub fk_column: String,
    pub foreign_table: String,
    pub foreign_column: String,
    pub on_update: ReferentialRule,
    pub on_delete: ReferentialRule,
}

/// Index definition
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexDefinition {
    pub name: String,
    pub columns: Vec<String>,
    pub is_unique: bool,
    pub is_primary: bool,
    pub ignore_nulls: bool,
}

/// Table definition
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableDefinition {
    pub name: String,
    pub columns: Vec<ColumnDefinition>,
    pub indexes: Vec<IndexDefinition>,
    pub foreign_keys: Vec<ForeignKeyDefinition>,
    pub description: Option<String>,
}

/// Schema validation report
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchemaValidationReport {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub table_count: usize,
    pub fk_count: usize,
}

/// Access Schema Engine managing tables, DDL generation, and validation
#[derive(Debug, Clone, Default)]
pub struct AccessSchemaEngine {
    pub tables: HashMap<String, TableDefinition>,
}

impl AccessSchemaEngine {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
        }
    }

    pub fn add_table(&mut self, table: TableDefinition) -> Result<()> {
        if self.tables.contains_key(&table.name) {
            return Err(TagisanError::Execution(format!(
                "Table '{}' is already defined in the schema",
                table.name
            )));
        }
        self.tables.insert(table.name.clone(), table);
        Ok(())
    }

    /// Validates referential integrity, primary keys, and circular dependencies
    pub fn validate(&self) -> SchemaValidationReport {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut total_fks = 0;

        for (tname, table) in &self.tables {
            let pk_count = table.columns.iter().filter(|c| c.is_primary_key).count();
            if pk_count == 0 {
                warnings.push(format!("Table '{}' has no primary key defined", tname));
            }

            let mut col_names = HashSet::new();
            for col in &table.columns {
                if !col_names.insert(&col.name) {
                    errors.push(format!("Duplicate column '{}' in table '{}'", col.name, tname));
                }
            }

            for fk in &table.foreign_keys {
                total_fks += 1;
                let parent_table = match self.tables.get(&fk.foreign_table) {
                    Some(pt) => pt,
                    None => {
                        errors.push(format!(
                            "Foreign key '{}' in table '{}' references non-existent table '{}'",
                            fk.name, tname, fk.foreign_table
                        ));
                        continue;
                    }
                };

                let child_col = table.columns.iter().find(|c| c.name == fk.fk_column);
                if child_col.is_none() {
                    errors.push(format!(
                        "Foreign key '{}' refers to non-existent local column '{}' in table '{}'",
                        fk.name, fk.fk_column, tname
                    ));
                }

                let parent_col = parent_table.columns.iter().find(|c| c.name == fk.foreign_column);
                if parent_col.is_none() {
                    errors.push(format!(
                        "Foreign key '{}' refers to non-existent foreign column '{}.{}'",
                        fk.name, fk.foreign_table, fk.foreign_column
                    ));
                }

                if let (Some(cc), Some(pc)) = (child_col, parent_col) {
                    if pc.data_type == AccessDataType::AutoNumber && cc.data_type != AccessDataType::LongInteger {
                        errors.push(format!(
                            "Type mismatch in FK '{}': parent '{}.{}' is AutoNumber, but child '{}.{}' is {:?}. Must be LongInteger.",
                            fk.name, fk.foreign_table, fk.foreign_column, tname, fk.fk_column, cc.data_type
                        ));
                    }
                }
            }
        }

        if let Err(cycle_err) = self.check_circular_dependencies() {
            errors.push(cycle_err);
        }

        SchemaValidationReport {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            table_count: self.tables.len(),
            fk_count: total_fks,
        }
    }

    fn check_circular_dependencies(&self) -> std::result::Result<(), String> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();

        for tname in self.tables.keys() {
            in_degree.insert(tname.clone(), 0);
            adj.insert(tname.clone(), Vec::new());
        }

        for (child_name, table) in &self.tables {
            for fk in &table.foreign_keys {
                if fk.foreign_table != *child_name && self.tables.contains_key(&fk.foreign_table) {
                    adj.get_mut(&fk.foreign_table).unwrap().push(child_name.clone());
                    *in_degree.get_mut(child_name).unwrap() += 1;
                }
            }
        }

        let mut queue: VecDeque<String> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(k, _)| k.clone())
            .collect();

        let mut visited_count = 0;
        while let Some(u) = queue.pop_front() {
            visited_count += 1;
            if let Some(neighbors) = adj.get(&u) {
                for v in neighbors {
                    let deg = in_degree.get_mut(v).unwrap();
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(v.clone());
                    }
                }
            }
        }

        if visited_count < self.tables.len() {
            let cyclic_tables: Vec<_> = in_degree
                .into_iter()
                .filter(|(_, deg)| *deg > 0)
                .map(|(k, _)| k)
                .collect();
            Err(format!(
                "Circular foreign key dependency cycle detected among tables: [{}]",
                cyclic_tables.join(", ")
            ))
        } else {
            Ok(())
        }
    }

    /// Generates complete ACE DDL script
    pub fn generate_ddl(&self) -> Result<String> {
        let validation = self.validate();
        if !validation.is_valid {
            return Err(TagisanError::Execution(format!(
                "Cannot generate DDL for invalid schema: {}",
                validation.errors.join("; ")
            )));
        }

        let mut ddl = String::new();
        ddl.push_str("-- =========================================================================\n");
        ddl.push_str("-- Microsoft Access 365 ACE / Jet 4.0 DDL Schema Script\n");
        ddl.push_str(&format!("-- Generated by Tagisan (tgs) at {}\n", Utc::now().to_rfc3339()));
        ddl.push_str("-- =========================================================================\n\n");

        for (tname, table) in &self.tables {
            ddl.push_str(&format!("CREATE TABLE [{}] (\n", tname));

            let mut col_defs = Vec::new();
            for col in &table.columns {
                let mut def = format!("    [{}] {}", col.name, col.data_type.to_ace_ddl());

                if !col.is_nullable {
                    def.push_str(" NOT NULL");
                }
                if col.is_primary_key {
                    def.push_str(" PRIMARY KEY");
                }
                if let Some(default) = &col.default_value {
                    def.push_str(&format!(" DEFAULT {}", default));
                }
                col_defs.push(def);
            }

            ddl.push_str(&col_defs.join(",\n"));
            ddl.push_str("\n);\n\n");

            for idx in &table.indexes {
                if idx.is_primary {
                    continue;
                }
                let unique_str = if idx.is_unique { "UNIQUE " } else { "" };
                let col_list = idx
                    .columns
                    .iter()
                    .map(|c| format!("[{}] ASC", c))
                    .collect::<Vec<_>>()
                    .join(", ");
                ddl.push_str(&format!(
                    "CREATE {}INDEX [{}] ON [{}] ({});\n",
                    unique_str, idx.name, tname, col_list
                ));
            }
            if !table.indexes.is_empty() {
                ddl.push('\n');
            }
        }

        ddl.push_str("-- =========================================================================\n");
        ddl.push_str("-- Referential Integrity Foreign Key Constraints\n");
        ddl.push_str("-- =========================================================================\n");

        for (tname, table) in &self.tables {
            for fk in &table.foreign_keys {
                ddl.push_str(&format!(
                    "ALTER TABLE [{}] ADD CONSTRAINT [{}] FOREIGN KEY ([{}]) REFERENCES [{}] ([{}]) ON UPDATE {} ON DELETE {};\n",
                    tname,
                    fk.name,
                    fk.fk_column,
                    fk.foreign_table,
                    fk.foreign_column,
                    fk.on_update.to_ddl(),
                    fk.on_delete.to_ddl()
                ));
            }
        }

        Ok(ddl)
    }

    /// Generates DAO Schema Builder VBA Code
    pub fn generate_dao_vba_builder(&self) -> String {
        let mut vba = String::new();
        vba.push_str("' =========================================================================\n");
        vba.push_str("' VBA DAO Schema Automation Module for Microsoft Access 365\n");
        vba.push_str("' Generated by Tagisan (tgs)\n");
        vba.push_str("' =========================================================================\n");
        vba.push_str("Option Compare Database\nOption Explicit\n\n");
        vba.push_str("Public Sub BuildDatabaseSchema()\n");
        vba.push_str("    Dim db As DAO.Database\n");
        vba.push_str("    Dim tdf As DAO.TableDef\n");
        vba.push_str("    Dim fld As DAO.Field\n");
        vba.push_str("    Dim rel As DAO.Relation\n\n");
        vba.push_str("    Set db = CurrentDb\n\n");

        for (tname, table) in &self.tables {
            vba.push_str(&format!("    ' Create Table [{}]\n", tname));
            vba.push_str(&format!("    Set tdf = db.CreateTableDef(\"{}\")\n", tname));
            for col in &table.columns {
                let dao_type = match col.data_type {
                    AccessDataType::ShortText { .. } => "dbText",
                    AccessDataType::LongText => "dbMemo",
                    AccessDataType::Byte => "dbByte",
                    AccessDataType::Integer => "dbInteger",
                    AccessDataType::LongInteger => "dbLong",
                    AccessDataType::Currency => "dbCurrency",
                    AccessDataType::SingleFloat => "dbSingle",
                    AccessDataType::DoubleFloat => "dbDouble",
                    AccessDataType::DateTime => "dbDate",
                    AccessDataType::YesNo => "dbBoolean",
                    AccessDataType::AutoNumber => "dbLong",
                    AccessDataType::Guid => "dbGUID",
                    AccessDataType::OleObject => "dbLongBinary",
                    AccessDataType::Hyperlink => "dbMemo",
                    AccessDataType::Decimal { .. } => "dbDecimal",
                };

                let size_param = if let AccessDataType::ShortText { max_length } = col.data_type {
                    format!(", {}", max_length)
                } else {
                    String::new()
                };

                vba.push_str(&format!(
                    "    Set fld = tdf.CreateField(\"{}\", {}{})\n",
                    col.name, dao_type, size_param
                ));
                if col.data_type == AccessDataType::AutoNumber {
                    vba.push_str("    fld.Attributes = dbAutoIncrField\n");
                }
                if !col.is_nullable {
                    vba.push_str("    fld.Required = True\n");
                }
                vba.push_str("    tdf.Fields.Append fld\n");
            }
            vba.push_str("    db.TableDefs.Append tdf\n\n");
        }

        vba.push_str("    db.TableDefs.Refresh\n");
        vba.push_str("    MsgBox \"Tagisan Access Schema Built Successfully!\", vbInformation, \"Tagisan Engine\"\n");
        vba.push_str("End Sub\n");

        vba
    }
}

// =========================================================================
// SECTION 3: AccessFormReportGenerator (SaveAsText twip geometry)
// =========================================================================

pub const TWIPS_PER_INCH: i32 = 1440;
pub const TWIPS_PER_CM: i32 = 567;
pub const TWIPS_PER_POINT: i32 = 20;

/// Form Control Types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FormReportControl {
    TextBox {
        name: String,
        control_source: String,
        left: i32,
        top: i32,
        width: i32,
        height: i32,
        font_name: String,
        font_size: u16,
        format: Option<String>,
        running_sum: Option<u8>, // 1 = Over All, 2 = Over Group
    },
    ComboBox {
        name: String,
        control_source: String,
        row_source: String,
        column_count: u8,
        column_widths: String,
        bound_column: u8,
        left: i32,
        top: i32,
        width: i32,
        height: i32,
    },
    CommandButton {
        name: String,
        caption: String,
        on_click: String,
        left: i32,
        top: i32,
        width: i32,
        height: i32,
    },
    Label {
        name: String,
        caption: String,
        left: i32,
        top: i32,
        width: i32,
        height: i32,
        font_size: u16,
        font_weight: u16, // 400 = Normal, 700 = Bold
    },
    CheckBox {
        name: String,
        control_source: String,
        left: i32,
        top: i32,
        width: i32,
        height: i32,
    },
}

/// Access Form Section (Detail, Header, Footer)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FormSection {
    pub name: String,
    pub height: i32,
    pub controls: Vec<FormReportControl>,
}

/// Form Definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AccessFormDefinition {
    pub name: String,
    pub record_source: String,
    pub caption: String,
    pub width: i32,
    pub sections: Vec<FormSection>,
}

/// Banded Report Section (ReportHeader, PageHeader, Detail, ReportFooter)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReportSection {
    pub name: String,
    pub height: i32,
    pub controls: Vec<FormReportControl>,
}

/// Banded Report Definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AccessReportDefinition {
    pub name: String,
    pub record_source: String,
    pub caption: String,
    pub width: i32,
    pub sections: Vec<ReportSection>,
}

/// Generates compliant SaveAsText / LoadFromText plain text representations
#[derive(Debug, Clone, Default)]
pub struct AccessFormReportGenerator;

impl AccessFormReportGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generates Microsoft Access Form in official `SaveAsText` format
    pub fn generate_form_save_as_text(&self, form: &AccessFormDefinition) -> String {
        let mut out = String::new();
        out.push_str("Version =20\n");
        out.push_str("VersionRequired =20\n");
        out.push_str("Checksum =-184204218\n");
        out.push_str("Begin Form\n");
        out.push_str(&format!("    RecordSource =\"{}\"\n", form.record_source));
        out.push_str(&format!("    Caption =\"{}\"\n", form.caption));
        out.push_str("    DefaultView =0\n");
        out.push_str("    ViewsAllowed =1\n");
        out.push_str("    GridX =24\n");
        out.push_str("    GridY =24\n");
        out.push_str(&format!("    Width ={}\n", form.width));
        out.push_str("    ItemSuffix =12\n");

        for section in &form.sections {
            out.push_str("    Begin Section\n");
            out.push_str(&format!("        Name =\"{}\"\n", section.name));
            out.push_str(&format!("        Height ={}\n", section.height));

            for ctrl in &section.controls {
                self.format_control(&mut out, ctrl, "        ");
            }

            out.push_str("    End\n");
        }

        out.push_str("End\n");
        out
    }

    /// Generates Microsoft Access Banded Report in official `SaveAsText` format
    pub fn generate_report_save_as_text(&self, report: &AccessReportDefinition) -> String {
        let mut out = String::new();
        out.push_str("Version =20\n");
        out.push_str("VersionRequired =20\n");
        out.push_str("Checksum =849102381\n");
        out.push_str("Begin Report\n");
        out.push_str(&format!("    RecordSource =\"{}\"\n", report.record_source));
        out.push_str(&format!("    Caption =\"{}\"\n", report.caption));
        out.push_str(&format!("    Width ={}\n", report.width));
        out.push_str("    GridX =24\n");
        out.push_str("    GridY =24\n");
        out.push_str("    ItemSuffix =16\n");

        for section in &report.sections {
            out.push_str("    Begin Section\n");
            out.push_str(&format!("        Name =\"{}\"\n", section.name));
            out.push_str(&format!("        Height ={}\n", section.height));

            for ctrl in &section.controls {
                self.format_control(&mut out, ctrl, "        ");
            }

            out.push_str("    End\n");
        }

        out.push_str("End\n");
        out
    }

    fn format_control(&self, out: &mut String, ctrl: &FormReportControl, indent: &str) {
        match ctrl {
            FormReportControl::TextBox {
                name,
                control_source,
                left,
                top,
                width,
                height,
                font_name,
                font_size,
                format,
                running_sum,
            } => {
                out.push_str(&format!("{}Begin TextBox\n", indent));
                out.push_str(&format!("{}    Name =\"{}\"\n", indent, name));
                out.push_str(&format!("{}    ControlSource =\"{}\"\n", indent, control_source));
                out.push_str(&format!("{}    Left ={}\n", indent, left));
                out.push_str(&format!("{}    Top ={}\n", indent, top));
                out.push_str(&format!("{}    Width ={}\n", indent, width));
                out.push_str(&format!("{}    Height ={}\n", indent, height));
                out.push_str(&format!("{}    FontName =\"{}\"\n", indent, font_name));
                out.push_str(&format!("{}    FontSize ={}\n", indent, font_size));
                if let Some(fmt) = format {
                    out.push_str(&format!("{}    Format =\"{}\"\n", indent, fmt));
                }
                if let Some(rs) = running_sum {
                    out.push_str(&format!("{}    RunningSum ={}\n", indent, rs));
                }
                out.push_str(&format!("{}End\n", indent));
            }
            FormReportControl::ComboBox {
                name,
                control_source,
                row_source,
                column_count,
                column_widths,
                bound_column,
                left,
                top,
                width,
                height,
            } => {
                out.push_str(&format!("{}Begin ComboBox\n", indent));
                out.push_str(&format!("{}    Name =\"{}\"\n", indent, name));
                out.push_str(&format!("{}    ControlSource =\"{}\"\n", indent, control_source));
                out.push_str(&format!("{}    RowSource =\"{}\"\n", indent, row_source));
                out.push_str(&format!("{}    ColumnCount ={}\n", indent, column_count));
                out.push_str(&format!("{}    ColumnWidths =\"{}\"\n", indent, column_widths));
                out.push_str(&format!("{}    BoundColumn ={}\n", indent, bound_column));
                out.push_str(&format!("{}    Left ={}\n", indent, left));
                out.push_str(&format!("{}    Top ={}\n", indent, top));
                out.push_str(&format!("{}    Width ={}\n", indent, width));
                out.push_str(&format!("{}    Height ={}\n", indent, height));
                out.push_str(&format!("{}End\n", indent));
            }
            FormReportControl::CommandButton {
                name,
                caption,
                on_click,
                left,
                top,
                width,
                height,
            } => {
                out.push_str(&format!("{}Begin CommandButton\n", indent));
                out.push_str(&format!("{}    Name =\"{}\"\n", indent, name));
                out.push_str(&format!("{}    Caption =\"{}\"\n", indent, caption));
                out.push_str(&format!("{}    OnClick =\"{}\"\n", indent, on_click));
                out.push_str(&format!("{}    Left ={}\n", indent, left));
                out.push_str(&format!("{}    Top ={}\n", indent, top));
                out.push_str(&format!("{}    Width ={}\n", indent, width));
                out.push_str(&format!("{}    Height ={}\n", indent, height));
                out.push_str(&format!("{}End\n", indent));
            }
            FormReportControl::Label {
                name,
                caption,
                left,
                top,
                width,
                height,
                font_size,
                font_weight,
            } => {
                out.push_str(&format!("{}Begin Label\n", indent));
                out.push_str(&format!("{}    Name =\"{}\"\n", indent, name));
                out.push_str(&format!("{}    Caption =\"{}\"\n", indent, caption));
                out.push_str(&format!("{}    Left ={}\n", indent, left));
                out.push_str(&format!("{}    Top ={}\n", indent, top));
                out.push_str(&format!("{}    Width ={}\n", indent, width));
                out.push_str(&format!("{}    Height ={}\n", indent, height));
                out.push_str(&format!("{}    FontSize ={}\n", indent, font_size));
                out.push_str(&format!("{}    FontWeight ={}\n", indent, font_weight));
                out.push_str(&format!("{}End\n", indent));
            }
            FormReportControl::CheckBox {
                name,
                control_source,
                left,
                top,
                width,
                height,
            } => {
                out.push_str(&format!("{}Begin CheckBox\n", indent));
                out.push_str(&format!("{}    Name =\"{}\"\n", indent, name));
                out.push_str(&format!("{}    ControlSource =\"{}\"\n", indent, control_source));
                out.push_str(&format!("{}    Left ={}\n", indent, left));
                out.push_str(&format!("{}    Top ={}\n", indent, top));
                out.push_str(&format!("{}    Width ={}\n", indent, width));
                out.push_str(&format!("{}    Height ={}\n", indent, height));
                out.push_str(&format!("{}End\n", indent));
            }
        }
    }
}

// =========================================================================
// SECTION 4: AccessVbaBridge (64-bit PtrSafe, WinHTTP, DAO Transactions)
// =========================================================================

/// Generates complete 64-bit PtrSafe VBA module
#[derive(Debug, Clone, Default)]
pub struct AccessVbaBridge {
    pub endpoint_url: String,
}

impl AccessVbaBridge {
    pub fn new() -> Self {
        Self {
            endpoint_url: "http://localhost:3978/api/copilot/access".to_string(),
        }
    }

    pub fn with_endpoint(mut self, url: impl Into<String>) -> Self {
        self.endpoint_url = url.into();
        self
    }

    /// Generates full production `.bas` module text
    pub fn generate_vba_module(&self) -> String {
        let mut vba = String::new();
        vba.push_str("Attribute VB_Name = \"modTagisanCopilot\"\n");
        vba.push_str("Option Compare Database\n");
        vba.push_str("Option Explicit\n\n");

        vba.push_str("' =========================================================================\n");
        vba.push_str("' Microsoft Access 365 64-Bit PtrSafe Win32/Win64 C-ABI Declarations\n");
        vba.push_str("' Generated by Tagisan Platform Architect\n");
        vba.push_str("' =========================================================================\n");
        vba.push_str("#If VBA7 Then\n");
        vba.push_str("    Private Declare PtrSafe Function GetCurrentProcessId Lib \"kernel32\" () As Long\n");
        vba.push_str("    Private Declare PtrSafe Function QueryPerformanceCounter Lib \"kernel32\" (lpPerformanceCount As Currency) As Long\n");
        vba.push_str("    Private Declare PtrSafe Function QueryPerformanceFrequency Lib \"kernel32\" (lpFrequency As Currency) As Long\n");
        vba.push_str("    Private Declare PtrSafe Sub Sleep Lib \"kernel32\" (ByVal dwMilliseconds As Long)\n");
        vba.push_str("#Else\n");
        vba.push_str("    Private Declare Function GetCurrentProcessId Lib \"kernel32\" () As Long\n");
        vba.push_str("    Private Declare Function QueryPerformanceCounter Lib \"kernel32\" (lpPerformanceCount As Currency) As Long\n");
        vba.push_str("    Private Declare Function QueryPerformanceFrequency Lib \"kernel32\" (lpFrequency As Currency) As Long\n");
        vba.push_str("    Private Declare Sub Sleep Lib \"kernel32\" (ByVal dwMilliseconds As Long)\n");
        vba.push_str("#End If\n\n");

        vba.push_str("Private Const TAGISAN_ENDPOINT As String = \"");
        vba.push_str(&self.endpoint_url);
        vba.push_str("\"\n\n");

        vba.push_str("' -------------------------------------------------------------------------\n");
        vba.push_str("' High-performance WinHTTP REST client invoking Tagisan Copilot Gateway\n");
        vba.push_str("' -------------------------------------------------------------------------\n");
        vba.push_str("Public Function TagisanCallCopilot(ByVal action As String, ByVal payloadJson As String, Optional ByVal purviewLabel As String = \"General\") As String\n");
        vba.push_str("    Dim http As Object\n");
        vba.push_str("    Dim envelope As String\n");
        vba.push_str("    Dim status As Long\n");
        vba.push_str("    On Error GoTo ErrHandler\n\n");
        vba.push_str("    Set http = CreateObject(\"MSXML2.ServerXMLHTTP.6.0\")\n");
        vba.push_str("    http.setTimeouts 5000, 10000, 30000, 30000\n\n");
        vba.push_str("    envelope = \"{\"\"action\"\":\"\"\" & action & \"\"\",\"\"purview_label\"\":\"\"\" & purviewLabel & \"\"\",\"\"payload\"\":\" & payloadJson & \"}\"\n\n");
        vba.push_str("    http.Open \"POST\", TAGISAN_ENDPOINT, False\n");
        vba.push_str("    http.setRequestHeader \"Content-Type\", \"application/json\"\n");
        vba.push_str("    http.setRequestHeader \"X-Tagisan-Purview-Label\", purviewLabel\n");
        vba.push_str("    http.setRequestHeader \"X-Client-Platform\", \"Microsoft Access 365 Win64\"\n");
        vba.push_str("    http.send envelope\n\n");
        vba.push_str("    status = http.status\n");
        vba.push_str("    If status = 200 Then\n");
        vba.push_str("        TagisanCallCopilot = http.responseText\n");
        vba.push_str("    Else\n");
        vba.push_str("        TagisanCallCopilot = \"{\"\"error\"\": true, \"\"code\"\": \" & status & \", \"\"message\"\": \"\"HTTP Error \" & status & \"\"\"}\"\n");
        vba.push_str("    End If\n");
        vba.push_str("    Exit Function\n\n");
        vba.push_str("ErrHandler:\n");
        vba.push_str("    TagisanCallCopilot = \"{\"\"error\"\": true, \"\"message\"\": \"\"\" & Replace(Err.Description, \"\"\"\", \"'\" ) & \"\"\"}\"\n");
        vba.push_str("End Function\n\n");

        vba.push_str("' -------------------------------------------------------------------------\n");
        vba.push_str("' DAO Workspace Transaction Execution with Force-OS Disk Flush\n");
        vba.push_str("' -------------------------------------------------------------------------\n");
        vba.push_str("Public Function TagisanExecuteDaoTransaction(ByVal sqlList As Variant) As Boolean\n");
        vba.push_str("    Dim ws As DAO.Workspace\n");
        vba.push_str("    Dim db As DAO.Database\n");
        vba.push_str("    Dim i As Long\n");
        vba.push_str("    On Error GoTo ErrHandler\n\n");
        vba.push_str("    Set ws = DBEngine.Workspaces(0)\n");
        vba.push_str("    Set db = ws.Databases(0)\n\n");
        vba.push_str("    ws.BeginTrans\n");
        vba.push_str("    For i = LBound(sqlList) To UBound(sqlList)\n");
        vba.push_str("        db.Execute CStr(sqlList(i)), dbFailOnError\n");
        vba.push_str("    Next i\n\n");
        vba.push_str("    ws.CommitTrans dbForceOSFlush\n");
        vba.push_str("    TagisanExecuteDaoTransaction = True\n");
        vba.push_str("    Exit Function\n\n");
        vba.push_str("ErrHandler:\n");
        vba.push_str("    ws.Rollback\n");
        vba.push_str("    TagisanExecuteDaoTransaction = False\n");
        vba.push_str("    MsgBox \"DAO Transaction Rolled Back: \" & Err.Description, vbCritical, \"Tagisan DAO Error\"\n");
        vba.push_str("End Function\n\n");

        vba.push_str("Public Function TagisanExecuteSql(ByVal sql As String) As Long\n");
        vba.push_str("    Dim db As DAO.Database\n");
        vba.push_str("    Set db = CurrentDb\n");
        vba.push_str("    db.Execute sql, dbFailOnError\n");
        vba.push_str("    TagisanExecuteSql = db.RecordsAffected\n");
        vba.push_str("End Function\n");

        vba
    }
}

// =========================================================================
// SECTION 5: AccessDataverseMigrator (Desktop Access -> Cloud Dataverse)
// =========================================================================

/// Cloud Dataverse Entity Migration Definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DataverseMigrationPlan {
    pub entity_logical_name: String,
    pub entity_display_name: String,
    pub entity_schema_json: Value,
    pub relationships_json: Vec<Value>,
    pub linked_table_connection_string: String,
    pub vba_linking_script: String,
}

/// Migrates desktop Access tables to cloud Microsoft Dataverse entities
#[derive(Debug, Clone, Default)]
pub struct AccessDataverseMigrator;

impl AccessDataverseMigrator {
    pub fn new() -> Self {
        Self
    }

    /// Generates cloud Dataverse entity metadata and ODBC linked table script
    pub fn plan_migration(
        &self,
        table: &TableDefinition,
        org_url: &str,
        table_prefix: Option<&str>,
    ) -> Result<DataverseMigrationPlan> {
        let prefix = table_prefix.unwrap_or("cr8bf");
        let entity_logical_name = format!("{}_{}", prefix, table.name.to_lowercase());
        let entity_display_name = table.name.clone();

        let mut attributes = Vec::new();
        let primary_id_attr = format!("{}_{}id", prefix, table.name.to_lowercase());
        let primary_name_attr = format!("{}_name", prefix);

        for col in &table.columns {
            let attr_schema = self.map_column_to_dataverse_attribute(prefix, col)?;
            attributes.push(attr_schema);
        }

        let entity_schema = json!({
            "SchemaName": format!("{}_{}", prefix, table.name),
            "LogicalName": entity_logical_name,
            "DisplayName": {
                "LocalizedLabels": [{ "Label": entity_display_name, "LanguageCode": 1033 }]
            },
            "DisplayCollectionName": {
                "LocalizedLabels": [{ "Label": format!("{}s", entity_display_name), "LanguageCode": 1033 }]
            },
            "Description": {
                "LocalizedLabels": [{ "Label": format!("Migrated from Access 365 table {}", table.name), "LanguageCode": 1033 }]
            },
            "OwnershipType": "UserOwned",
            "IsActivity": false,
            "HasNotes": true,
            "Attributes": attributes,
            "PrimaryIdAttribute": primary_id_attr,
            "PrimaryNameAttribute": primary_name_attr
        });

        let mut relationships = Vec::new();
        for fk in &table.foreign_keys {
            let rel_schema = json!({
                "SchemaName": format!("{}_{}_{}", prefix, fk.foreign_table.to_lowercase(), table.name.to_lowercase()),
                "ReferencedEntity": format!("{}_{}", prefix, fk.foreign_table.to_lowercase()),
                "ReferencingEntity": entity_logical_name,
                "ReferencingAttribute": format!("{}_{}", prefix, fk.fk_column.to_lowercase()),
                "CascadeConfiguration": {
                    "Assign": "NoCascade",
                    "Delete": if fk.on_delete == ReferentialRule::Cascade { "Cascade" } else { "RemoveLink" },
                    "Merge": "NoCascade",
                    "Read": "NoCascade",
                    "Reparent": "NoCascade",
                    "Share": "NoCascade",
                    "Unshare": "NoCascade"
                }
            });
            relationships.push(rel_schema);
        }

        let org_clean = org_url.trim_start_matches("https://").trim_matches('/');
        let conn_str = format!(
            "ODBC;Driver={{ODBC Driver 18 for SQL Server}};Server={};Database={};Authentication=ActiveDirectoryInteractive;Encrypt=yes;TrustServerCertificate=no;",
            org_clean,
            org_clean.split('.').next().unwrap_or("Dataverse")
        );

        let vba_script = format!(
            "Public Sub LinkDataverse_{}()\n\
            \x20   Dim db As DAO.Database\n\
            \x20   Dim tdf As DAO.TableDef\n\
            \x20   Set db = CurrentDb\n\
            \x20   On Error Resume Next\n\
            \x20   db.TableDefs.Delete \"{}\"\n\
            \x20   On Error GoTo 0\n\
            \x20   Set tdf = db.CreateTableDef(\"{}\")\n\
            \x20   tdf.Connect = \"{}\"\n\
            \x20   tdf.SourceTableName = \"{}\"\n\
            \x20   db.TableDefs.Append tdf\n\
            \x20   db.TableDefs.Refresh\n\
            \x20   MsgBox \"Dataverse Entity '{}' successfully linked to Access!\", vbInformation, \"Tagisan Migrator\"\n\
            End Sub\n",
            table.name, table.name, table.name, conn_str, entity_logical_name, table.name
        );

        Ok(DataverseMigrationPlan {
            entity_logical_name,
            entity_display_name,
            entity_schema_json: entity_schema,
            relationships_json: relationships,
            linked_table_connection_string: conn_str,
            vba_linking_script: vba_script,
        })
    }

    fn map_column_to_dataverse_attribute(&self, prefix: &str, col: &ColumnDefinition) -> Result<Value> {
        let logical_name = format!("{}_{}", prefix, col.name.to_lowercase());
        let display_name = col.name.clone();

        let (type_str, format_opt, extra) = match &col.data_type {
            AccessDataType::ShortText { max_length } => (
                "StringAttributeMetadata",
                Some("Text"),
                json!({ "MaxLength": max_length }),
            ),
            AccessDataType::LongText => (
                "MemoAttributeMetadata",
                Some("TextArea"),
                json!({ "MaxLength": 1048576 }),
            ),
            AccessDataType::Byte | AccessDataType::Integer | AccessDataType::LongInteger | AccessDataType::AutoNumber => (
                "IntegerAttributeMetadata",
                None,
                json!({ "MinValue": -2147483648i32, "MaxValue": 2147483647i32 }),
            ),
            AccessDataType::Currency => (
                "MoneyAttributeMetadata",
                None,
                json!({ "Precision": 4, "MinValue": -922337203685477.0, "MaxValue": 922337203685477.0 }),
            ),
            AccessDataType::SingleFloat | AccessDataType::DoubleFloat => (
                "DoubleAttributeMetadata",
                None,
                json!({ "MinValue": -100000000000.0, "MaxValue": 100000000000.0 }),
            ),
            AccessDataType::DateTime => (
                "DateTimeAttributeMetadata",
                Some("DateAndTime"),
                json!({ "DateTimeBehavior": { "Value": "UserLocal" } }),
            ),
            AccessDataType::YesNo => (
                "BooleanAttributeMetadata",
                None,
                json!({
                    "OptionSet": {
                        "TrueOption": { "Value": 1, "Label": { "LocalizedLabels": [{ "Label": "Yes", "LanguageCode": 1033 }] } },
                        "FalseOption": { "Value": 0, "Label": { "LocalizedLabels": [{ "Label": "No", "LanguageCode": 1033 }] } }
                    }
                }),
            ),
            AccessDataType::Guid => (
                "UniqueIdentifierAttributeMetadata",
                None,
                json!({}),
            ),
            _ => (
                "StringAttributeMetadata",
                Some("Text"),
                json!({ "MaxLength": 255 }),
            ),
        };

        let mut val = json!({
            "@odata.type": format!("Microsoft.Dynamics.CRM.{}", type_str),
            "SchemaName": format!("{}_{}", prefix, col.name),
            "LogicalName": logical_name,
            "DisplayName": {
                "LocalizedLabels": [{ "Label": display_name, "LanguageCode": 1033 }]
            },
            "RequiredLevel": {
                "Value": if col.is_nullable { "None" } else { "ApplicationRequired" }
            }
        });

        if let Some(fmt) = format_opt {
            val["Format"] = json!(fmt);
        }

        if let (Some(obj), Some(extra_obj)) = (val.as_object_mut(), extra.as_object()) {
            for (k, v) in extra_obj {
                obj.insert(k.clone(), v.clone());
            }
        }

        Ok(val)
    }
}

// =========================================================================
// SECTION 6: AccessBlastRadiusAnalyzer (Traces Table/Column Renames)
// =========================================================================

/// Affected component location
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlastImpactItem {
    pub component_type: String,
    pub component_name: String,
    pub property_or_line: String,
    pub matched_snippet: String,
    pub suggested_patch: String,
}

/// Blast radius assessment report
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccessBlastRadiusReport {
    pub target_symbol: String,
    pub target_new_name: String,
    pub total_impacted_items: usize,
    pub risk_level: String,
    pub impacted_queries: Vec<BlastImpactItem>,
    pub impacted_forms: Vec<BlastImpactItem>,
    pub impacted_reports: Vec<BlastImpactItem>,
    pub impacted_vba_modules: Vec<BlastImpactItem>,
    pub surgical_remediation_script: String,
}

/// Analyzes blast radius of schema alterations across Access assets
#[derive(Debug, Clone, Default)]
pub struct AccessBlastRadiusAnalyzer;

impl AccessBlastRadiusAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Traces target table or column rename across queries, forms, reports, and VBA code
    pub fn analyze(
        &self,
        old_symbol: &str,
        new_symbol: &str,
        queries: &[(&str, &str)],
        forms: &[&AccessFormDefinition],
        reports: &[&AccessReportDefinition],
        vba_modules: &[(&str, &str)],
    ) -> AccessBlastRadiusReport {
        let mut impacted_queries = Vec::new();
        let mut impacted_forms = Vec::new();
        let mut impacted_reports = Vec::new();
        let mut impacted_vba = Vec::new();

        let pattern = format!(r"(?i)\b{}\b", regex::escape(old_symbol));
        let re = Regex::new(&pattern).unwrap();

        for (qname, sql) in queries {
            if re.is_match(sql) {
                let patched_sql = re.replace_all(sql, new_symbol).to_string();
                impacted_queries.push(BlastImpactItem {
                    component_type: "Query".to_string(),
                    component_name: qname.to_string(),
                    property_or_line: "SQL".to_string(),
                    matched_snippet: sql.trim().to_string(),
                    suggested_patch: patched_sql,
                });
            }
        }

        for form in forms {
            if re.is_match(&form.record_source) {
                impacted_forms.push(BlastImpactItem {
                    component_type: "Form".to_string(),
                    component_name: form.name.clone(),
                    property_or_line: "RecordSource".to_string(),
                    matched_snippet: form.record_source.clone(),
                    suggested_patch: re.replace_all(&form.record_source, new_symbol).to_string(),
                });
            }

            for section in &form.sections {
                for ctrl in &section.controls {
                    match ctrl {
                        FormReportControl::TextBox { name, control_source, .. }
                        | FormReportControl::CheckBox { name, control_source, .. } => {
                            if re.is_match(control_source) {
                                impacted_forms.push(BlastImpactItem {
                                    component_type: "Form".to_string(),
                                    component_name: form.name.clone(),
                                    property_or_line: format!("Control '{}'.ControlSource", name),
                                    matched_snippet: control_source.clone(),
                                    suggested_patch: re.replace_all(control_source, new_symbol).to_string(),
                                });
                            }
                        }
                        FormReportControl::ComboBox { name, control_source, row_source, .. } => {
                            if re.is_match(control_source) {
                                impacted_forms.push(BlastImpactItem {
                                    component_type: "Form".to_string(),
                                    component_name: form.name.clone(),
                                    property_or_line: format!("Control '{}'.ControlSource", name),
                                    matched_snippet: control_source.clone(),
                                    suggested_patch: re.replace_all(control_source, new_symbol).to_string(),
                                });
                            }
                            if re.is_match(row_source) {
                                impacted_forms.push(BlastImpactItem {
                                    component_type: "Form".to_string(),
                                    component_name: form.name.clone(),
                                    property_or_line: format!("Control '{}'.RowSource", name),
                                    matched_snippet: row_source.clone(),
                                    suggested_patch: re.replace_all(row_source, new_symbol).to_string(),
                                });
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        for report in reports {
            if re.is_match(&report.record_source) {
                impacted_reports.push(BlastImpactItem {
                    component_type: "Report".to_string(),
                    component_name: report.name.clone(),
                    property_or_line: "RecordSource".to_string(),
                    matched_snippet: report.record_source.clone(),
                    suggested_patch: re.replace_all(&report.record_source, new_symbol).to_string(),
                });
            }

            for section in &report.sections {
                for ctrl in &section.controls {
                    if let FormReportControl::TextBox { name, control_source, .. } = ctrl {
                        if re.is_match(control_source) {
                            impacted_reports.push(BlastImpactItem {
                                component_type: "Report".to_string(),
                                component_name: report.name.clone(),
                                property_or_line: format!("Control '{}'.ControlSource", name),
                                matched_snippet: control_source.clone(),
                                suggested_patch: re.replace_all(control_source, new_symbol).to_string(),
                            });
                        }
                    }
                }
            }
        }

        for (mname, code) in vba_modules {
            for (line_idx, line) in code.lines().enumerate() {
                if re.is_match(line) {
                    impacted_vba.push(BlastImpactItem {
                        component_type: "VBA".to_string(),
                        component_name: mname.to_string(),
                        property_or_line: format!("Line {}", line_idx + 1),
                        matched_snippet: line.trim().to_string(),
                        suggested_patch: re.replace_all(line, new_symbol).to_string(),
                    });
                }
            }
        }

        let total = impacted_queries.len() + impacted_forms.len() + impacted_reports.len() + impacted_vba.len();
        let risk = if total == 0 {
            "Low"
        } else if total <= 3 && impacted_vba.is_empty() {
            "Medium"
        } else if total <= 10 {
            "High"
        } else {
            "Critical"
        };

        let mut script = String::new();
        script.push_str("' =========================================================================\n");
        script.push_str(&format!("' Tagisan Surgical Migration Script: Renaming '{}' -> '{}'\n", old_symbol, new_symbol));
        script.push_str(&format!("' Risk Rating: {} | Impacted Artifacts: {}\n", risk, total));
        script.push_str("' =========================================================================\n\n");

        for q in &impacted_queries {
            script.push_str(&format!("' Update Query [{}]\n", q.component_name));
            script.push_str(&format!("CurrentDb.QueryDefs(\"{}\").SQL = \"{}\"\n\n", q.component_name, q.suggested_patch.replace('"', "\"\"")));
        }

        AccessBlastRadiusReport {
            target_symbol: old_symbol.to_string(),
            target_new_name: new_symbol.to_string(),
            total_impacted_items: total,
            risk_level: risk.to_string(),
            impacted_queries,
            impacted_forms,
            impacted_reports,
            impacted_vba_modules: impacted_vba,
            surgical_remediation_script: script,
        }
    }
}

// =========================================================================
// SECTION 7: CopilotAccessTool (Implements ToolHandler)
// =========================================================================

/// Tool exposing Microsoft Access 365 capabilities to autonomous agents
pub struct CopilotAccessTool {
    transpiler: AceSqlTranspiler,
    vba_bridge: AccessVbaBridge,
    dataverse_migrator: AccessDataverseMigrator,
    blast_analyzer: AccessBlastRadiusAnalyzer,
}

impl Default for CopilotAccessTool {
    fn default() -> Self {
        Self::new()
    }
}

impl CopilotAccessTool {
    pub fn new() -> Self {
        Self {
            transpiler: AceSqlTranspiler::new(),
            vba_bridge: AccessVbaBridge::new(),
            dataverse_migrator: AccessDataverseMigrator::new(),
            blast_analyzer: AccessBlastRadiusAnalyzer::new(),
        }
    }
}

#[async_trait]
impl ToolHandler for CopilotAccessTool {
    fn name(&self) -> &str {
        "copilot_access"
    }

    fn description(&self) -> &str {
        "Enterprise Microsoft Access 365 integration subsystem: transpiles ANSI SQL to strict Jet/ACE SQL with parenthesized joins, generates table schemas and DDL, produces SaveAsText forms and banded reports with twip coordinates, creates 64-bit PtrSafe VBA REST bridges, migrates Access to Dataverse, and computes blast radius analysis."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": [
                        "transpile_sql",
                        "generate_schema",
                        "generate_form",
                        "generate_report",
                        "generate_vba",
                        "dataverse_migrate",
                        "blast_radius",
                        "dlp_audit"
                    ],
                    "description": "Access 365 subsystem operation to perform"
                },
                "sql": {
                    "type": "string",
                    "description": "ANSI SQL query to transpile to Jet/ACE SQL or audit"
                },
                "schema_definition": {
                    "type": "object",
                    "description": "JSON representation of table definitions, columns, and foreign keys"
                },
                "form_definition": {
                    "type": "object",
                    "description": "JSON representation of form and control coordinates"
                },
                "report_definition": {
                    "type": "object",
                    "description": "JSON representation of banded report sections and running sums"
                },
                "old_symbol": {
                    "type": "string",
                    "description": "Symbol being renamed for blast radius analysis"
                },
                "new_symbol": {
                    "type": "string",
                    "description": "New replacement symbol name"
                },
                "target_url": {
                    "type": "string",
                    "description": "Dataverse organization URL for cloud migration"
                },
                "purview_label": {
                    "type": "string",
                    "enum": ["General", "Confidential", "HighlyConfidential", "Secret"],
                    "description": "Purview data sensitivity label"
                }
            },
            "required": ["action"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let action = arguments
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing required parameter 'action'".to_string()))?;

        let arg_str = arguments.to_string();
        let dlp_verdict = AgentShieldScanner::scan_outbound_dlp(&arg_str);
        if let AgentShieldVerdict::Block { reason, threat_level } = dlp_verdict {
            return Err(TagisanError::Security(format!(
                "AgentShield Outbound DLP Gate blocked Access 365 request (Threat: {:?}): {}",
                threat_level, reason
            )));
        }

        match action {
            "transpile_sql" => {
                let sql = arguments
                    .get("sql")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| TagisanError::Execution("Missing parameter 'sql'".to_string()))?;

                let res = self.transpiler.transpile(sql)?;
                let mut out = format!(
                    "### 🗄️ Microsoft Access 365 / ACE Transpiled SQL\n\n\
                    - **Join Nesting Depth:** {}\n\
                    - **Cartesian Product Detected:** {}\n\
                    - **Purview Classification:** {}\n\
                    - **Tables Detected:** {}\n\n\
                    ```sql\n{}\n```",
                    res.join_depth,
                    if res.is_cartesian { "⚠️ YES" } else { "✅ No" },
                    res.purview_sensitivity.badge(),
                    res.tables_referenced.join(", "),
                    res.transpiled_sql
                );

                if !res.warnings.is_empty() {
                    out.push_str("\n\n**Warnings:**\n");
                    for w in res.warnings {
                        out.push_str(&format!("- ⚠️ {}\n", w));
                    }
                }

                Ok(out)
            }

            "generate_schema" => {
                let schema_val = arguments
                    .get("schema_definition")
                    .ok_or_else(|| TagisanError::Execution("Missing parameter 'schema_definition'".to_string()))?;

                let tables: Vec<TableDefinition> = serde_json::from_value(schema_val.clone())
                    .map_err(|e| TagisanError::Execution(format!("Invalid schema_definition: {}", e)))?;

                let mut engine = AccessSchemaEngine::new();
                for t in tables {
                    engine.add_table(t)?;
                }

                let ddl = engine.generate_ddl()?;
                let vba = engine.generate_dao_vba_builder();

                Ok(format!(
                    "### 🗄️ Microsoft Access 365 ACE Schema DDL & DAO Builder\n\n\
                    #### ACE DDL Script\n```sql\n{}\n```\n\n\
                    #### DAO Automation VBA Module\n```vba\n{}\n```",
                    ddl, vba
                ))
            }

            "generate_form" => {
                let form_val = arguments
                    .get("form_definition")
                    .ok_or_else(|| TagisanError::Execution("Missing parameter 'form_definition'".to_string()))?;

                let form: AccessFormDefinition = serde_json::from_value(form_val.clone())
                    .map_err(|e| TagisanError::Execution(format!("Invalid form_definition: {}", e)))?;

                let gen = AccessFormReportGenerator::new();
                let txt = gen.generate_form_save_as_text(&form);

                Ok(format!(
                    "### 📑 Microsoft Access Form SaveAsText Format (Twip Coordinates)\n\n\
                    - **Form Name:** `{}`\n\
                    - **Width (twips):** {} (approx {:.2} inches)\n\n\
                    ```access\n{}\n```",
                    form.name,
                    form.width,
                    form.width as f32 / TWIPS_PER_INCH as f32,
                    txt
                ))
            }

            "generate_report" => {
                let report_val = arguments
                    .get("report_definition")
                    .ok_or_else(|| TagisanError::Execution("Missing parameter 'report_definition'".to_string()))?;

                let report: AccessReportDefinition = serde_json::from_value(report_val.clone())
                    .map_err(|e| TagisanError::Execution(format!("Invalid report_definition: {}", e)))?;

                let gen = AccessFormReportGenerator::new();
                let txt = gen.generate_report_save_as_text(&report);

                Ok(format!(
                    "### 📊 Microsoft Access Banded Report SaveAsText Format\n\n\
                    - **Report Name:** `{}`\n\
                    - **Sections:** {}\n\n\
                    ```access\n{}\n```",
                    report.name,
                    report.sections.iter().map(|s| s.name.as_str()).collect::<Vec<_>>().join(", "),
                    txt
                ))
            }

            "generate_vba" => {
                let vba = self.vba_bridge.generate_vba_module();
                Ok(format!(
                    "### 💻 Microsoft Access 365 64-bit PtrSafe VBA REST & DAO Module\n\n\
                    ```vba\n{}\n```",
                    vba
                ))
            }

            "dataverse_migrate" => {
                let schema_val = arguments
                    .get("schema_definition")
                    .ok_or_else(|| TagisanError::Execution("Missing parameter 'schema_definition'".to_string()))?;

                let table: TableDefinition = serde_json::from_value(schema_val.clone())
                    .map_err(|e| TagisanError::Execution(format!("Invalid table definition: {}", e)))?;

                let org_url = arguments
                    .get("target_url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("myorg.crm.dynamics.com");

                let plan = self.dataverse_migrator.plan_migration(&table, org_url, None)?;

                Ok(format!(
                    "### ☁️ Microsoft Access 365 to Cloud Dataverse Migration Plan\n\n\
                    - **Dataverse Entity Logical Name:** `{}`\n\
                    - **ODBC Linked Table Connection:** `{}`\n\n\
                    #### Dataverse Web API Entity Schema\n```json\n{}\n```\n\n\
                    #### Access Linked Table Automation Script\n```vba\n{}\n```",
                    plan.entity_logical_name,
                    plan.linked_table_connection_string,
                    serde_json::to_string_pretty(&plan.entity_schema_json).unwrap_or_default(),
                    plan.vba_linking_script
                ))
            }

            "blast_radius" => {
                let old_sym = arguments
                    .get("old_symbol")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| TagisanError::Execution("Missing parameter 'old_symbol'".to_string()))?;

                let new_sym = arguments
                    .get("new_symbol")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| TagisanError::Execution("Missing parameter 'new_symbol'".to_string()))?;

                let queries_raw = arguments.get("queries").and_then(|v| v.as_array());
                let mut queries_vec = Vec::new();
                if let Some(arr) = queries_raw {
                    for item in arr {
                        if let (Some(name), Some(sql)) = (item.get("name").and_then(|v| v.as_str()), item.get("sql").and_then(|v| v.as_str())) {
                            queries_vec.push((name, sql));
                        }
                    }
                }

                let report = self.blast_analyzer.analyze(old_sym, new_sym, &queries_vec, &[], &[], &[]);

                Ok(format!(
                    "### 💥 Microsoft Access 365 Blast Radius Analysis\n\n\
                    - **Target Symbol:** `{}` ➔ `{}`\n\
                    - **Risk Level:** **{}**\n\
                    - **Total Affected Assets:** {}\n\
                    - **Impacted Queries:** {}\n\n\
                    #### Surgical Remediation Patch\n```vba\n{}\n```",
                    report.target_symbol,
                    report.target_new_name,
                    report.risk_level,
                    report.total_impacted_items,
                    report.impacted_queries.len(),
                    report.surgical_remediation_script
                ))
            }

            "dlp_audit" => {
                let content = arguments
                    .get("sql")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();

                let purview_label = arguments
                    .get("purview_label")
                    .and_then(|v| v.as_str());

                let res = PurviewGuardEngine::evaluate(content, purview_label, None)?;

                Ok(format!(
                    "### 🛡️ Microsoft Access 365 Purview & AgentShield DLP Audit\n\n\
                    - **Sensitivity:** {}\n\
                    - **Zero-Egress Air-Gapped:** {}\n\
                    - **Receipt ID:** `{}`\n\
                    - **SHA-256 Digest:** `{}`\n\
                    - **Signature:** `{}`",
                    res.sensitivity.badge(),
                    if res.air_gapped { "🔒 Enforced" } else { "🟢 Not Required" },
                    res.receipt.receipt_id,
                    res.receipt.content_sha256,
                    res.receipt.signature_sha256
                ))
            }

            other => Err(TagisanError::Execution(format!(
                "Unknown action '{}'. Expected 'transpile_sql', 'generate_schema', 'generate_form', 'generate_report', 'generate_vba', 'dataverse_migrate', 'blast_radius', or 'dlp_audit'",
                other
            ))),
        }
    }
}
