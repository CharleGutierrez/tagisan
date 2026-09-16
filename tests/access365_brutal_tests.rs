//! # Dedicated Brutal Test Suite for Microsoft Access 365 Subsystem
//!
//! Exhaustive test suite covering:
//! - Multi-table parenthesized join transpilation (depth 1, 2, 3, 4)
//! - Function conversions: COALESCE -> Nz, CASE WHEN -> nested IIf, string concat || -> &, Mid, Len, Now
//! - Date literals #yyyy-mm-dd# and timestamp conversions
//! - Cartesian product detection and O(N*M) risk warnings
//! - Schema Engine DDL generation (all Access data types)
//! - Referential integrity verification, foreign keys, cascade rules, and circular dependency detection
//! - Form & Banded Report SaveAsText twip geometry (1440 twips = 1 inch) and running sums
//! - 64-bit PtrSafe VBA REST client & DAO transaction wrappers (dbForceOSFlush)
//! - Cloud Dataverse entity migration schema mapping and ODBC linked tables
//! - AST Blast Radius cross-component analysis (queries, forms, reports, VBA modules)
//! - Purview Zero-Egress Air-Gap enforcement and AgentShield DLP defenses
//! - Full CopilotAccessTool ToolHandler execution
//! - 50-worker concurrent async stress testing

use std::sync::Arc;
use tagisan::copilot::access::*;
use tagisan::copilot::purview::PurviewSensitivity;
use tagisan::tools::ToolHandler;
use serde_json::json;

#[test]
fn test_01_transpiler_nested_joins_three_and_four_tables() {
    let transpiler = AceSqlTranspiler::new();

    // 1. Three-table join (depth 2)
    let sql_3 = "SELECT o.OrderID, c.CompanyName, e.LastName \
                 FROM Orders AS o \
                 INNER JOIN Customers AS c ON o.CustomerID = c.CustomerID \
                 LEFT JOIN Employees AS e ON o.EmployeeID = e.EmployeeID \
                 WHERE o.OrderID > 100";

    let res_3 = transpiler.transpile(sql_3).expect("Transpilation failed");
    assert_eq!(res_3.join_depth, 2);
    assert!(!res_3.is_cartesian);
    assert!(res_3.transpiled_sql.contains("((Orders AS o INNER JOIN Customers AS c ON o.CustomerID = c.CustomerID) LEFT JOIN Employees AS e ON o.EmployeeID = e.EmployeeID)"));

    // 2. Four-table join (depth 3)
    let sql_4 = "SELECT o.OrderID, c.CompanyName, e.LastName, s.ShipperName \
                 FROM Orders AS o \
                 INNER JOIN Customers AS c ON o.CustomerID = c.CustomerID \
                 LEFT JOIN Employees AS e ON o.EmployeeID = e.EmployeeID \
                 RIGHT JOIN Shippers AS s ON o.ShipperID = s.ShipperID \
                 ORDER BY o.OrderID DESC";

    let res_4 = transpiler.transpile(sql_4).expect("Transpilation failed");
    assert_eq!(res_4.join_depth, 3);
    assert!(!res_4.is_cartesian);
    assert!(res_4.transpiled_sql.contains("(((Orders AS o INNER JOIN Customers AS c ON o.CustomerID = c.CustomerID) LEFT JOIN Employees AS e ON o.EmployeeID = e.EmployeeID) RIGHT JOIN Shippers AS s ON o.ShipperID = s.ShipperID)"));
}

#[test]
fn test_02_transpiler_coalesce_and_functions() {
    let transpiler = AceSqlTranspiler::new();

    // 1. Two-arg COALESCE
    let sql_two = "SELECT COALESCE(NickName, FirstName) AS DisplayName FROM Employees";
    let res_two = transpiler.transpile(sql_two).expect("Transpilation failed");
    assert!(res_two.transpiled_sql.contains("Nz(NickName, FirstName)"));

    // 2. Multi-arg COALESCE -> nested Nz()
    let sql_multi = "SELECT COALESCE(MobilePhone, HomePhone, WorkPhone, 'None') AS ContactPhone FROM Contacts";
    let res_multi = transpiler.transpile(sql_multi).expect("Transpilation failed");
    assert!(res_multi.transpiled_sql.contains("Nz(MobilePhone, Nz(HomePhone, Nz(WorkPhone, 'None'))"));

    // 3. String concat || -> &
    let sql_concat = "SELECT FirstName || ' ' || LastName AS FullName FROM Employees";
    let res_concat = transpiler.transpile(sql_concat).expect("Transpilation failed");
    assert!(res_concat.transpiled_sql.contains("FirstName & ' ' & LastName"));

    // 4. SUBSTRING -> Mid, LENGTH -> Len, CURRENT_TIMESTAMP -> Now()
    let sql_funcs = "SELECT SUBSTRING(Notes FROM 1 FOR 50) AS Snippet, LENGTH(Notes) AS NoteLen, CURRENT_TIMESTAMP AS CheckedAt FROM Logs";
    let res_funcs = transpiler.transpile(sql_funcs).expect("Transpilation failed");
    assert!(res_funcs.transpiled_sql.contains("Mid(Notes, 1, 50)"));
    assert!(res_funcs.transpiled_sql.contains("Len(Notes)"));
    assert!(res_funcs.transpiled_sql.contains("Now()"));
}

#[test]
fn test_03_transpiler_case_when_to_nested_iif() {
    let transpiler = AceSqlTranspiler::new();

    // 1. Simple CASE WHEN with ELSE
    let sql_case = "SELECT OrderID, CASE WHEN Status = 1 THEN 'New' WHEN Status = 2 THEN 'Processing' WHEN Status = 3 THEN 'Shipped' ELSE 'Cancelled' END AS StatusDesc FROM Orders";
    let res_case = transpiler.transpile(sql_case).expect("Transpilation failed");
    assert!(res_case.transpiled_sql.contains("IIf(Status = 1, 'New', IIf(Status = 2, 'Processing', IIf(Status = 3, 'Shipped', 'Cancelled'))"));

    // 2. CASE WHEN without ELSE (should default to Null)
    let sql_no_else = "SELECT CASE WHEN Score >= 90 THEN 'A' WHEN Score >= 80 THEN 'B' END AS Grade FROM Students";
    let res_no_else = transpiler.transpile(sql_no_else).expect("Transpilation failed");
    assert!(res_no_else.transpiled_sql.contains("IIf(Score >= 90, 'A', IIf(Score >= 80, 'B', Null))"));
}

#[test]
fn test_04_transpiler_date_literals_and_timestamps() {
    let transpiler = AceSqlTranspiler::new();

    // 1. DATE keyword literal
    let sql_date = "SELECT * FROM Orders WHERE OrderDate >= DATE '2026-09-16'";
    let res_date = transpiler.transpile(sql_date).expect("Transpilation failed");
    assert!(res_date.transpiled_sql.contains("#2026-09-16#"));

    // 2. ISO Date in comparison
    let sql_iso = "SELECT * FROM Invoices WHERE DueDate < '2026-12-31' AND PaidDate = '2026-10-15'";
    let res_iso = transpiler.transpile(sql_iso).expect("Transpilation failed");
    assert!(res_iso.transpiled_sql.contains("#2026-12-31#"));
    assert!(res_iso.transpiled_sql.contains("#2026-10-15#"));

    // 3. Timestamp literal
    let sql_ts = "SELECT * FROM Events WHERE EventTime >= '2026-09-16 14:30:00'";
    let res_ts = transpiler.transpile(sql_ts).expect("Transpilation failed");
    assert!(res_ts.transpiled_sql.contains("#2026-09-16 14:30:00#"));
}

#[test]
fn test_05_transpiler_cartesian_product_detection() {
    let transpiler = AceSqlTranspiler::new();

    // 1. Unbounded comma join without WHERE
    let sql_cart = "SELECT * FROM Orders, Customers, Products";
    let res_cart = transpiler.transpile(sql_cart).expect("Transpilation failed");
    assert!(res_cart.is_cartesian);
    assert!(!res_cart.warnings.is_empty());
    assert!(res_cart.warnings[0].contains("Cartesian product detected"));

    // 2. Explicit CROSS JOIN
    let sql_cross = "SELECT * FROM Categories CROSS JOIN Suppliers";
    let res_cross = transpiler.transpile(sql_cross).expect("Transpilation failed");
    assert!(res_cross.is_cartesian);

    // 3. Proper join with ON clause (not Cartesian)
    let sql_proper = "SELECT * FROM Orders INNER JOIN Customers ON Orders.CustomerID = Customers.CustomerID";
    let res_proper = transpiler.transpile(sql_proper).expect("Transpilation failed");
    assert!(!res_proper.is_cartesian);
}

#[test]
fn test_06_schema_engine_ddl_generation() {
    let mut engine = AccessSchemaEngine::new();

    // Parent table: Customers
    let customers = TableDefinition {
        name: "Customers".to_string(),
        columns: vec![
            ColumnDefinition {
                name: "CustomerID".to_string(),
                data_type: AccessDataType::AutoNumber,
                is_primary_key: true,
                is_nullable: false,
                is_unique: true,
                default_value: None,
                validation_rule: None,
                validation_text: None,
                description: Some("Primary Customer AutoNumber ID".to_string()),
            },
            ColumnDefinition {
                name: "CompanyName".to_string(),
                data_type: AccessDataType::ShortText { max_length: 100 },
                is_primary_key: false,
                is_nullable: false,
                is_unique: false,
                default_value: None,
                validation_rule: None,
                validation_text: None,
                description: None,
            },
            ColumnDefinition {
                name: "IsActive".to_string(),
                data_type: AccessDataType::YesNo,
                is_primary_key: false,
                is_nullable: false,
                is_unique: false,
                default_value: Some("True".to_string()),
                validation_rule: None,
                validation_text: None,
                description: None,
            },
        ],
        indexes: vec![
            IndexDefinition {
                name: "idx_CompanyName".to_string(),
                columns: vec!["CompanyName".to_string()],
                is_unique: false,
                is_primary: false,
                ignore_nulls: false,
            }
        ],
        foreign_keys: vec![],
        description: Some("Customer master table".to_string()),
    };

    // Child table: Orders
    let orders = TableDefinition {
        name: "Orders".to_string(),
        columns: vec![
            ColumnDefinition {
                name: "OrderID".to_string(),
                data_type: AccessDataType::AutoNumber,
                is_primary_key: true,
                is_nullable: false,
                is_unique: true,
                default_value: None,
                validation_rule: None,
                validation_text: None,
                description: None,
            },
            ColumnDefinition {
                name: "CustomerID".to_string(),
                data_type: AccessDataType::LongInteger,
                is_primary_key: false,
                is_nullable: false,
                is_unique: false,
                default_value: None,
                validation_rule: None,
                validation_text: None,
                description: None,
            },
            ColumnDefinition {
                name: "OrderDate".to_string(),
                data_type: AccessDataType::DateTime,
                is_primary_key: false,
                is_nullable: true,
                is_unique: false,
                default_value: Some("Now()".to_string()),
                validation_rule: None,
                validation_text: None,
                description: None,
            },
            ColumnDefinition {
                name: "OrderTotal".to_string(),
                data_type: AccessDataType::Currency,
                is_primary_key: false,
                is_nullable: false,
                is_unique: false,
                default_value: Some("0.00".to_string()),
                validation_rule: None,
                validation_text: None,
                description: None,
            },
            ColumnDefinition {
                name: "SpecialInstructions".to_string(),
                data_type: AccessDataType::LongText,
                is_primary_key: false,
                is_nullable: true,
                is_unique: false,
                default_value: None,
                validation_rule: None,
                validation_text: None,
                description: None,
            },
        ],
        indexes: vec![],
        foreign_keys: vec![
            ForeignKeyDefinition {
                name: "FK_Orders_Customers".to_string(),
                fk_column: "CustomerID".to_string(),
                foreign_table: "Customers".to_string(),
                foreign_column: "CustomerID".to_string(),
                on_update: ReferentialRule::Cascade,
                on_delete: ReferentialRule::Cascade,
            }
        ],
        description: Some("Orders transaction table".to_string()),
    };

    engine.add_table(customers).unwrap();
    engine.add_table(orders).unwrap();

    let validation = engine.validate();
    assert!(validation.is_valid, "Validation errors: {:?}", validation.errors);
    assert_eq!(validation.table_count, 2);
    assert_eq!(validation.fk_count, 1);

    let ddl = engine.generate_ddl().expect("DDL generation failed");
    assert!(ddl.contains("CREATE TABLE [Customers]"));
    assert!(ddl.contains("[CustomerID] AUTOINCREMENT NOT NULL PRIMARY KEY"));
    assert!(ddl.contains("[CompanyName] TEXT(100) NOT NULL"));
    assert!(ddl.contains("[IsActive] YESNO NOT NULL DEFAULT True"));
    assert!(ddl.contains("CREATE INDEX [idx_CompanyName] ON [Customers] ([CompanyName] ASC);"));

    assert!(ddl.contains("CREATE TABLE [Orders]"));
    assert!(ddl.contains("[OrderTotal] CURRENCY NOT NULL DEFAULT 0.00"));
    assert!(ddl.contains("[SpecialInstructions] MEMO"));
    assert!(ddl.contains("ALTER TABLE [Orders] ADD CONSTRAINT [FK_Orders_Customers] FOREIGN KEY ([CustomerID]) REFERENCES [Customers] ([CustomerID]) ON UPDATE CASCADE ON DELETE CASCADE;"));
}

#[test]
fn test_07_schema_referential_integrity_and_cycle_detection() {
    // 1. Missing target table
    let mut engine = AccessSchemaEngine::new();
    let broken_fk_table = TableDefinition {
        name: "Payments".to_string(),
        columns: vec![
            ColumnDefinition {
                name: "PaymentID".to_string(),
                data_type: AccessDataType::AutoNumber,
                is_primary_key: true,
                is_nullable: false,
                is_unique: true,
                default_value: None,
                validation_rule: None,
                validation_text: None,
                description: None,
            },
            ColumnDefinition {
                name: "InvoiceID".to_string(),
                data_type: AccessDataType::LongInteger,
                is_primary_key: false,
                is_nullable: false,
                is_unique: false,
                default_value: None,
                validation_rule: None,
                validation_text: None,
                description: None,
            },
        ],
        indexes: vec![],
        foreign_keys: vec![
            ForeignKeyDefinition {
                name: "FK_Payments_Invoices".to_string(),
                fk_column: "InvoiceID".to_string(),
                foreign_table: "NonExistentInvoices".to_string(),
                foreign_column: "InvoiceID".to_string(),
                on_update: ReferentialRule::Cascade,
                on_delete: ReferentialRule::Cascade,
            }
        ],
        description: None,
    };
    engine.add_table(broken_fk_table).unwrap();
    let val_missing = engine.validate();
    assert!(!val_missing.is_valid);
    assert!(val_missing.errors[0].contains("references non-existent table 'NonExistentInvoices'"));

    // 2. Circular dependency cycle between Table A and Table B
    let mut cycle_engine = AccessSchemaEngine::new();
    let table_a = TableDefinition {
        name: "TableA".to_string(),
        columns: vec![
            ColumnDefinition {
                name: "A_ID".to_string(),
                data_type: AccessDataType::LongInteger,
                is_primary_key: true,
                is_nullable: false,
                is_unique: true,
                default_value: None,
                validation_rule: None,
                validation_text: None,
                description: None,
            },
            ColumnDefinition {
                name: "B_ID".to_string(),
                data_type: AccessDataType::LongInteger,
                is_primary_key: false,
                is_nullable: false,
                is_unique: false,
                default_value: None,
                validation_rule: None,
                validation_text: None,
                description: None,
            },
        ],
        indexes: vec![],
        foreign_keys: vec![
            ForeignKeyDefinition {
                name: "FK_A_B".to_string(),
                fk_column: "B_ID".to_string(),
                foreign_table: "TableB".to_string(),
                foreign_column: "B_ID".to_string(),
                on_update: ReferentialRule::NoAction,
                on_delete: ReferentialRule::NoAction,
            }
        ],
        description: None,
    };

    let table_b = TableDefinition {
        name: "TableB".to_string(),
        columns: vec![
            ColumnDefinition {
                name: "B_ID".to_string(),
                data_type: AccessDataType::LongInteger,
                is_primary_key: true,
                is_nullable: false,
                is_unique: true,
                default_value: None,
                validation_rule: None,
                validation_text: None,
                description: None,
            },
            ColumnDefinition {
                name: "A_ID".to_string(),
                data_type: AccessDataType::LongInteger,
                is_primary_key: false,
                is_nullable: false,
                is_unique: false,
                default_value: None,
                validation_rule: None,
                validation_text: None,
                description: None,
            },
        ],
        indexes: vec![],
        foreign_keys: vec![
            ForeignKeyDefinition {
                name: "FK_B_A".to_string(),
                fk_column: "A_ID".to_string(),
                foreign_table: "TableA".to_string(),
                foreign_column: "A_ID".to_string(),
                on_update: ReferentialRule::NoAction,
                on_delete: ReferentialRule::NoAction,
            }
        ],
        description: None,
    };

    cycle_engine.add_table(table_a).unwrap();
    cycle_engine.add_table(table_b).unwrap();
    let val_cycle = cycle_engine.validate();
    assert!(!val_cycle.is_valid);
    assert!(val_cycle.errors.iter().any(|e| e.contains("Circular foreign key dependency cycle detected")));
}

#[test]
fn test_08_form_generator_twip_geometry_and_controls() {
    let gen = AccessFormReportGenerator::new();

    // 1 inch = 1440 twips
    let form = AccessFormDefinition {
        name: "frmCustomerDetail".to_string(),
        record_source: "Customers".to_string(),
        caption: "Customer Detail Record".to_string(),
        width: 8640,
        sections: vec![
            FormSection {
                name: "Detail".to_string(),
                height: 4320, // 3 inches
                controls: vec![
                    FormReportControl::Label {
                        name: "lblCompanyName".to_string(),
                        caption: "Company Name:".to_string(),
                        left: 720,
                        top: 720,
                        width: 1440,
                        height: 360,
                        font_size: 10,
                        font_weight: 700,
                    },
                    FormReportControl::TextBox {
                        name: "txtCompanyName".to_string(),
                        control_source: "CompanyName".to_string(),
                        left: 2304,
                        top: 720,
                        width: 4320,
                        height: 432,
                        font_name: "Segoe UI".to_string(),
                        font_size: 10,
                        format: None,
                        running_sum: None,
                    },
                    FormReportControl::ComboBox {
                        name: "cboCountry".to_string(),
                        control_source: "CountryID".to_string(),
                        row_source: "SELECT CountryID, CountryName FROM Countries ORDER BY CountryName;".to_string(),
                        column_count: 2,
                        column_widths: "0;2880".to_string(),
                        bound_column: 1,
                        left: 2304,
                        top: 1440,
                        width: 2880,
                        height: 432,
                    },
                    FormReportControl::CommandButton {
                        name: "cmdSave".to_string(),
                        caption: "&Save Record".to_string(),
                        on_click: "[Event Procedure]".to_string(),
                        left: 2304,
                        top: 2880,
                        width: 2160,
                        height: 576,
                    },
                ],
            }
        ],
    };

    let text = gen.generate_form_save_as_text(&form);
    assert!(text.contains("Version =20"));
    assert!(text.contains("Begin Form"));
    assert!(text.contains("RecordSource =\"Customers\""));
    assert!(text.contains("Width =8640"));
    assert!(text.contains("Begin Section"));
    assert!(text.contains("Name =\"Detail\""));
    assert!(text.contains("Height =4320"));

    // Check Controls
    assert!(text.contains("Begin Label"));
    assert!(text.contains("Caption =\"Company Name:\""));
    assert!(text.contains("Begin TextBox"));
    assert!(text.contains("Name =\"txtCompanyName\""));
    assert!(text.contains("ControlSource =\"CompanyName\""));
    assert!(text.contains("Begin ComboBox"));
    assert!(text.contains("ColumnWidths =\"0;2880\""));
    assert!(text.contains("Begin CommandButton"));
    assert!(text.contains("OnClick =\"[Event Procedure]\""));
}

#[test]
fn test_09_report_generator_banded_structure_and_running_sums() {
    let gen = AccessFormReportGenerator::new();

    let report = AccessReportDefinition {
        name: "rptQuarterlySummary".to_string(),
        record_source: "qryQuarterlyOrders".to_string(),
        caption: "Quarterly Sales Summary".to_string(),
        width: 10080,
        sections: vec![
            ReportSection {
                name: "ReportHeader".to_string(),
                height: 1440,
                controls: vec![
                    FormReportControl::Label {
                        name: "lblTitle".to_string(),
                        caption: "Executive Quarterly Sales".to_string(),
                        left: 720,
                        top: 360,
                        width: 5760,
                        height: 576,
                        font_size: 16,
                        font_weight: 700,
                    }
                ],
            },
            ReportSection {
                name: "Detail".to_string(),
                height: 432,
                controls: vec![
                    FormReportControl::TextBox {
                        name: "txtOrderDate".to_string(),
                        control_source: "OrderDate".to_string(),
                        left: 720,
                        top: 0,
                        width: 1440,
                        height: 432,
                        font_name: "Segoe UI".to_string(),
                        font_size: 9,
                        format: Some("Short Date".to_string()),
                        running_sum: None,
                    },
                    FormReportControl::TextBox {
                        name: "txtAmount".to_string(),
                        control_source: "TotalAmount".to_string(),
                        left: 4320,
                        top: 0,
                        width: 1800,
                        height: 432,
                        font_name: "Segoe UI".to_string(),
                        font_size: 9,
                        format: Some("Currency".to_string()),
                        running_sum: None,
                    },
                ],
            },
            ReportSection {
                name: "ReportFooter".to_string(),
                height: 1080,
                controls: vec![
                    FormReportControl::TextBox {
                        name: "txtGrandTotal".to_string(),
                        control_source: "=Sum([TotalAmount])".to_string(),
                        left: 4320,
                        top: 180,
                        width: 1800,
                        height: 432,
                        font_name: "Segoe UI".to_string(),
                        font_size: 10,
                        format: Some("Currency".to_string()),
                        running_sum: Some(1),
                    }
                ],
            },
        ],
    };

    let text = gen.generate_report_save_as_text(&report);
    assert!(text.contains("Begin Report"));
    assert!(text.contains("RecordSource =\"qryQuarterlyOrders\""));
    assert!(text.contains("Name =\"ReportHeader\""));
    assert!(text.contains("Name =\"Detail\""));
    assert!(text.contains("Name =\"ReportFooter\""));
    assert!(text.contains("ControlSource =\"=Sum([TotalAmount])\""));
    assert!(text.contains("RunningSum =1"));
}

#[test]
fn test_10_vba_bridge_64bit_ptrsafe_and_winhttp() {
    let bridge = AccessVbaBridge::new().with_endpoint("http://localhost:3978/api/copilot/access");
    let vba = bridge.generate_vba_module();

    assert!(vba.contains("Attribute VB_Name = \"modTagisanCopilot\""));
    assert!(vba.contains("#If VBA7 Then"));
    assert!(vba.contains("Private Declare PtrSafe Function GetCurrentProcessId Lib \"kernel32\" () As Long"));
    assert!(vba.contains("Private Declare PtrSafe Function QueryPerformanceCounter Lib \"kernel32\" (lpPerformanceCount As Currency) As Long"));
    assert!(vba.contains("MSXML2.ServerXMLHTTP.6.0"));
    assert!(vba.contains("http://localhost:3978/api/copilot/access"));
    assert!(vba.contains("X-Tagisan-Purview-Label"));
    assert!(vba.contains("ws.BeginTrans"));
    assert!(vba.contains("ws.CommitTrans dbForceOSFlush"));
    assert!(vba.contains("ws.Rollback"));
}

#[test]
fn test_11_dataverse_migrator_schema_and_linked_tables() {
    let migrator = AccessDataverseMigrator::new();

    let table = TableDefinition {
        name: "Invoices".to_string(),
        columns: vec![
            ColumnDefinition {
                name: "InvoiceID".to_string(),
                data_type: AccessDataType::AutoNumber,
                is_primary_key: true,
                is_nullable: false,
                is_unique: true,
                default_value: None,
                validation_rule: None,
                validation_text: None,
                description: None,
            },
            ColumnDefinition {
                name: "InvoiceNumber".to_string(),
                data_type: AccessDataType::ShortText { max_length: 50 },
                is_primary_key: false,
                is_nullable: false,
                is_unique: true,
                default_value: None,
                validation_rule: None,
                validation_text: None,
                description: None,
            },
            ColumnDefinition {
                name: "SubTotal".to_string(),
                data_type: AccessDataType::Currency,
                is_primary_key: false,
                is_nullable: false,
                is_unique: false,
                default_value: None,
                validation_rule: None,
                validation_text: None,
                description: None,
            },
            ColumnDefinition {
                name: "InvoiceDate".to_string(),
                data_type: AccessDataType::DateTime,
                is_primary_key: false,
                is_nullable: false,
                is_unique: false,
                default_value: None,
                validation_rule: None,
                validation_text: None,
                description: None,
            },
            ColumnDefinition {
                name: "IsPaid".to_string(),
                data_type: AccessDataType::YesNo,
                is_primary_key: false,
                is_nullable: false,
                is_unique: false,
                default_value: None,
                validation_rule: None,
                validation_text: None,
                description: None,
            },
        ],
        indexes: vec![],
        foreign_keys: vec![],
        description: None,
    };

    let plan = migrator
        .plan_migration(&table, "contoso.crm.dynamics.com", Some("cr8bf"))
        .expect("Migration planning failed");

    assert_eq!(plan.entity_logical_name, "cr8bf_invoices");
    assert_eq!(plan.entity_display_name, "Invoices");

    let attrs = plan.entity_schema_json.get("Attributes").unwrap().as_array().unwrap();
    assert_eq!(attrs.len(), 5);

    let money_attr = attrs.iter().find(|a| a.get("LogicalName").unwrap() == "cr8bf_subtotal").unwrap();
    assert_eq!(money_attr.get("@odata.type").unwrap(), "Microsoft.Dynamics.CRM.MoneyAttributeMetadata");

    assert!(plan.linked_table_connection_string.contains("Driver={ODBC Driver 18 for SQL Server}"));
    assert!(plan.linked_table_connection_string.contains("Server=contoso.crm.dynamics.com"));
    assert!(plan.linked_table_connection_string.contains("Authentication=ActiveDirectoryInteractive"));

    assert!(plan.vba_linking_script.contains("Public Sub LinkDataverse_Invoices()"));
    assert!(plan.vba_linking_script.contains("tdf.SourceTableName = \"cr8bf_invoices\""));
}

#[test]
fn test_12_blast_radius_analyzer_cross_component_tracing() {
    let analyzer = AccessBlastRadiusAnalyzer::new();

    let queries = [
        ("qryActiveOrders", "SELECT OrderID, CustomerID, OrderTotal FROM Orders WHERE IsCancelled = False"),
        ("qryCustomerSummary", "SELECT CustomerID, COUNT(OrderID) AS TotalOrders FROM Orders GROUP BY CustomerID"),
    ];

    let form = AccessFormDefinition {
        name: "frmOrders".to_string(),
        record_source: "Orders".to_string(),
        caption: "Order Entry".to_string(),
        width: 8640,
        sections: vec![
            FormSection {
                name: "Detail".to_string(),
                height: 2880,
                controls: vec![
                    FormReportControl::TextBox {
                        name: "txtCustID".to_string(),
                        control_source: "CustomerID".to_string(),
                        left: 1440,
                        top: 720,
                        width: 2880,
                        height: 432,
                        font_name: "Segoe UI".to_string(),
                        font_size: 10,
                        format: None,
                        running_sum: None,
                    }
                ],
            }
        ],
    };

    let report = AccessReportDefinition {
        name: "rptCustomerOrders".to_string(),
        record_source: "Orders".to_string(),
        caption: "Customer Report".to_string(),
        width: 10080,
        sections: vec![
            ReportSection {
                name: "Detail".to_string(),
                height: 432,
                controls: vec![
                    FormReportControl::TextBox {
                        name: "txtCustomer".to_string(),
                        control_source: "CustomerID".to_string(),
                        left: 720,
                        top: 0,
                        width: 1440,
                        height: 432,
                        font_name: "Segoe UI".to_string(),
                        font_size: 9,
                        format: None,
                        running_sum: None,
                    }
                ],
            }
        ],
    };

    let vba_modules = [
        ("modOrderProcessing", "Sub ProcessOrder(rs As DAO.Recordset)\n    Dim cId As Long\n    cId = rs!CustomerID\nEnd Sub"),
        ("modAudit", "Public Sub AuditOrders()\n    CurrentDb.Execute \"UPDATE Orders SET CustomerID = 999 WHERE CustomerID = 0\"\nEnd Sub"),
    ];

    let report_res = analyzer.analyze(
        "CustomerID",
        "ClientUID",
        &queries,
        &[&form],
        &[&report],
        &vba_modules,
    );

    assert_eq!(report_res.target_symbol, "CustomerID");
    assert_eq!(report_res.target_new_name, "ClientUID");
    assert_eq!(report_res.impacted_queries.len(), 2);
    assert_eq!(report_res.impacted_forms.len(), 1);
    assert_eq!(report_res.impacted_reports.len(), 1);
    assert_eq!(report_res.impacted_vba_modules.len(), 2);
    assert_eq!(report_res.total_impacted_items, 6);
    assert_eq!(report_res.risk_level, "High");
    assert!(report_res.surgical_remediation_script.contains("CurrentDb.QueryDefs(\"qryActiveOrders\").SQL"));
}

#[test]
fn test_13_purview_and_agentshield_dlp_defense() {
    let tool = CopilotAccessTool::new();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let malicious_arg = json!({
        "action": "transpile_sql",
        "sql": "SELECT * FROM Users WHERE PrivateKey = '-----BEGIN RSA PRIVATE KEY-----\nMIIEowIBAAKCAQEA0...' AND Active = 1"
    });

    let res_block = rt.block_on(tool.execute(malicious_arg));
    assert!(res_block.is_err());
    let err_str = res_block.unwrap_err().to_string();
    assert!(err_str.contains("AgentShield Outbound DLP Gate blocked Access 365 request") || err_str.contains("Security"));

    let confidential_arg = json!({
        "action": "transpile_sql",
        "sql": "SELECT EmployeeID, SalarySchedule, HipaaHealthData FROM ConfidentialEmployees WHERE SalarySchedule > 100000"
    });

    let res_conf = rt.block_on(tool.execute(confidential_arg)).unwrap();
    assert!(res_conf.contains("Highly Confidential") || res_conf.contains("Confidential"));
    assert!(res_conf.contains("Zero-Egress"));
}

#[test]
fn test_14_copilot_access_tool_end_to_end() {
    let tool = CopilotAccessTool::new();
    assert_eq!(tool.name(), "copilot_access");
    assert!(tool.description().contains("Microsoft Access 365"));

    let rt = tokio::runtime::Runtime::new().unwrap();

    let transpile_arg = json!({
        "action": "transpile_sql",
        "sql": "SELECT COALESCE(o.Notes, 'Standard') AS OrderNotes FROM Orders AS o INNER JOIN Customers AS c ON o.CustomerID = c.CustomerID WHERE o.OrderDate = DATE '2026-09-16'"
    });
    let out_transpile = rt.block_on(tool.execute(transpile_arg)).unwrap();
    assert!(out_transpile.contains("Nz(o.Notes, 'Standard')"));
    assert!(out_transpile.contains("#2026-09-16#"));

    let vba_arg = json!({
        "action": "generate_vba"
    });
    let out_vba = rt.block_on(tool.execute(vba_arg)).unwrap();
    assert!(out_vba.contains("TagisanCallCopilot"));
    assert!(out_vba.contains("TagisanExecuteDaoTransaction"));

    let blast_arg = json!({
        "action": "blast_radius",
        "old_symbol": "UnitPrice",
        "new_symbol": "BasePrice",
        "queries": [
            { "name": "qryProducts", "sql": "SELECT ProductID, UnitPrice FROM Products" }
        ]
    });
    let out_blast = rt.block_on(tool.execute(blast_arg)).unwrap();
    assert!(out_blast.contains("UnitPrice") && out_blast.contains("BasePrice"));
    assert!(out_blast.contains("**Impacted Queries:** 1"));
}

#[tokio::test]
async fn test_15_50_worker_concurrency_stress() {
    let transpiler = Arc::new(AceSqlTranspiler::new());
    let analyzer = Arc::new(AccessBlastRadiusAnalyzer::new());
    let mut handles = Vec::with_capacity(50);

    for worker_id in 0..50 {
        let t = Arc::clone(&transpiler);
        let a = Arc::clone(&analyzer);

        let handle = tokio::spawn(async move {
            let query = format!(
                "SELECT o.OrderID, COALESCE(c.CompanyName, 'Unknown'), \
                 CASE WHEN o.Status = 1 THEN 'Active' ELSE 'Pending' END \
                 FROM Orders_{} AS o \
                 INNER JOIN Customers_{} AS c ON o.CustomerID = c.CustomerID \
                 LEFT JOIN Shippers_{} AS s ON o.ShipperID = s.ShipperID \
                 WHERE o.CreatedDate >= DATE '2026-09-16'",
                worker_id, worker_id, worker_id
            );

            let res = t.transpile(&query).expect("Worker transpilation failed");
            assert_eq!(res.join_depth, 2);
            assert!(res.transpiled_sql.contains("Nz("));
            assert!(res.transpiled_sql.contains("IIf("));
            assert!(res.transpiled_sql.contains("#2026-09-16#"));

            let old_sym = format!("Orders_{}", worker_id);
            let new_sym = format!("SalesOrders_{}", worker_id);
            let queries = [
                ("qryWorker", query.as_str())
            ];
            let blast = a.analyze(&old_sym, &new_sym, &queries, &[], &[], &[]);
            assert_eq!(blast.impacted_queries.len(), 1);

            worker_id
        });

        handles.push(handle);
    }

    for (idx, handle) in handles.into_iter().enumerate() {
        let worker_res = handle.await.expect("Worker panicked or failed");
        assert_eq!(worker_res, idx);
    }
}
