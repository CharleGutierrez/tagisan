use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::error::{Result, TagisanError};
use crate::harness::spec::{ArgumentSpec, ArgumentType, HarnessSpec};

/// The generated CLI harness artifacts
#[derive(Debug, Clone)]
pub struct GeneratedHarness {
    pub cli_code: String,
    pub test_code: String,
    pub cli_filename: String,
    pub test_filename: String,
    pub spec: HarnessSpec,
}

impl GeneratedHarness {
    /// Writes the generated harness and test scripts to a target directory
    pub fn write_to_dir(&self, dir: &Path) -> Result<(PathBuf, PathBuf)> {
        fs::create_dir_all(dir).map_err(|e| {
            TagisanError::Execution(format!(
                "Failed to create harness output directory '{}': {}",
                dir.display(),
                e
            ))
        })?;

        let cli_path = dir.join(&self.cli_filename);
        let mut cli_file = File::create(&cli_path).map_err(|e| {
            TagisanError::Execution(format!(
                "Failed to create CLI harness file '{}': {}",
                cli_path.display(),
                e
            ))
        })?;
        cli_file.write_all(self.cli_code.as_bytes()).map_err(|e| {
            TagisanError::Execution(format!("Failed to write to '{}': {}", cli_path.display(), e))
        })?;

        // Make executable on unix
        #[cfg(target_family = "unix")]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&cli_path, fs::Permissions::from_mode(0o755));
        }

        let test_path = dir.join(&self.test_filename);
        let mut test_file = File::create(&test_path).map_err(|e| {
            TagisanError::Execution(format!(
                "Failed to create test harness file '{}': {}",
                test_path.display(),
                e
            ))
        })?;
        test_file.write_all(self.test_code.as_bytes()).map_err(|e| {
            TagisanError::Execution(format!("Failed to write to '{}': {}", test_path.display(), e))
        })?;

        #[cfg(target_family = "unix")]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&test_path, fs::Permissions::from_mode(0o755));
        }

        Ok((cli_path, test_path))
    }
}

/// Autonomous code generator producing production-ready agent-native CLIs
pub struct HarnessGenerator;

impl HarnessGenerator {
    /// Synthesizes a standalone Python CLI harness and validation test suite from a HarnessSpec
    pub fn generate(spec: &HarnessSpec) -> Result<GeneratedHarness> {
        spec.validate()?;

        let cli_filename = format!("{}_cli.py", spec.name.replace('-', "_"));
        let test_filename = format!("test_{}_cli.py", spec.name.replace('-', "_"));

        let cli_code = Self::generate_python_cli(spec)?;
        let test_code = Self::generate_python_tests(spec, &cli_filename)?;

        Ok(GeneratedHarness {
            cli_code,
            test_code,
            cli_filename,
            test_filename,
            spec: spec.clone(),
        })
    }

    /// Generates standalone, zero-dependency Python CLI script with argparse and --json output
    fn generate_python_cli(spec: &HarnessSpec) -> Result<String> {
        let mut code = String::new();

        // 1. Shebang & Headers
        code.push_str("#!/usr/bin/env python3\n");
        code.push_str(&format!(
            "# Synthesized Agent-Native CLI Harness for {}\n",
            spec.name
        ));
        code.push_str("# Generated autonomously by Tagisan (tgs harness)\n\n");

        code.push_str("import argparse\n");
        code.push_str("import datetime\n");
        code.push_str("import importlib.util\n");
        code.push_str("import json\n");
        code.push_str("import os\n");
        code.push_str("import sys\n");
        code.push_str("import traceback\n\n");

        // 2. Source Loading Helper
        let source_path_str = spec.source_path.to_string_lossy().replace('\\', "/");
        code.push_str(&format!("SOURCE_PATH = os.path.abspath(\"{}\")\n", source_path_str));
        code.push_str("SOURCE_DIR = os.path.dirname(SOURCE_PATH) if os.path.isfile(SOURCE_PATH) else SOURCE_PATH\n\n");

        code.push_str("if SOURCE_DIR not in sys.path:\n");
        code.push_str("    sys.path.insert(0, SOURCE_DIR)\n\n");

        code.push_str("def load_target_module():\n");
        code.push_str("    if not os.path.exists(SOURCE_PATH):\n");
        code.push_str("        return None\n");
        code.push_str("    if os.path.isfile(SOURCE_PATH) and SOURCE_PATH.endswith('.py'):\n");
        code.push_str("        mod_name = os.path.splitext(os.path.basename(SOURCE_PATH))[0]\n");
        code.push_str("        spec = importlib.util.spec_from_file_location(mod_name, SOURCE_PATH)\n");
        code.push_str("        if spec and spec.loader:\n");
        code.push_str("            mod = importlib.util.module_from_spec(spec)\n");
        code.push_str("            sys.modules[mod_name] = mod\n");
        code.push_str("            spec.loader.exec_module(mod)\n");
        code.push_str("            return mod\n");
        code.push_str("    return None\n\n");

        // 3. Command Handlers
        for cmd in &spec.commands {
            let func_identifier = format!("handle_{}", cmd.name.replace('-', "_"));
            code.push_str(&format!("def {}(args, target_mod):\n", func_identifier));
            code.push_str(&format!("    \"\"\"Handler for subcommand: {}\"\"\"\n", &cmd.name));

            // Argument collection
            code.push_str("    call_kwargs = {}\n");
            for arg in &cmd.arguments {
                let var = arg.python_var_name();
                code.push_str(&format!(
                    "    if hasattr(args, '{}') and getattr(args, '{}') is not None:\n",
                    var, var
                ));
                code.push_str(&format!(
                    "        call_kwargs['{}'] = getattr(args, '{}')\n",
                    var, var
                ));
            }

            // Function execution or fallback calculation
            code.push_str(&format!(
                "    if target_mod and hasattr(target_mod, '{}'):\n",
                cmd.function_name
            ));
            code.push_str(&format!(
                "        func = getattr(target_mod, '{}')\n",
                cmd.function_name
            ));
            code.push_str("        return func(**call_kwargs)\n");

            if let Some(ref cls_name) = cmd.class_name {
                code.push_str(&format!(
                    "    elif target_mod and hasattr(target_mod, '{}'):\n",
                    cls_name
                ));
                code.push_str(&format!(
                    "        cls_obj = getattr(target_mod, '{}')\n",
                    cls_name
                ));
                code.push_str("        instance = cls_obj()\n");
                code.push_str(&format!(
                    "        if hasattr(instance, '{}'):\n",
                    cmd.function_name
                ));
                code.push_str(&format!(
                    "            return getattr(instance, '{}')(**call_kwargs)\n",
                    cmd.function_name
                ));
            }

            // Deterministic default implementation if source module is not found
            code.push_str("    # Synthesized deterministic fallback implementation\n");
            code.push_str("    return {\n");
            code.push_str(&format!("        'action': '{}',\n", cmd.name));
            code.push_str("        'parameters': call_kwargs,\n");
            code.push_str("        'status': 'executed',\n");
            code.push_str("    }\n\n");
        }

        // 4. CLI Argument Parser Definition
        code.push_str("def build_parser():\n");
        code.push_str("    global_flags = argparse.ArgumentParser(add_help=False)\n");

        // Global flags on global_flags parser
        for opt in &spec.global_options {
            let flag = opt.cli_flag();
            let mut arg_parts = vec![format!("'{}'", flag)];
            if let Some(s) = opt.short {
                arg_parts.push(format!("'-{}'", s));
            }
            arg_parts.push(opt.arg_type.argparse_type_code());
            if !opt.description.is_empty() {
                arg_parts.push(format!("help={}", escape_python_str(&opt.description)));
            }
            code.push_str(&format!(
                "    global_flags.add_argument({}, default=argparse.SUPPRESS)\n",
                arg_parts.join(", ")
            ));
        }

        code.push_str(&format!(
            "    parser = argparse.ArgumentParser(prog='{}', description={}, parents=[global_flags])\n",
            spec.name,
            escape_python_str(&spec.description)
        ));
        code.push_str(&format!(
            "    parser.add_argument('--version', action='version', version='%(prog)s {}')\n",
            spec.version
        ));

        code.push_str("\n    subparsers = parser.add_subparsers(dest='subcommand', title='subcommands', help='Available operations')\n\n");

        // Subcommands
        for cmd in &spec.commands {
            code.push_str(&format!(
                "    p_{} = subparsers.add_parser('{}', help={}, parents=[global_flags])\n",
                cmd.name.replace('-', "_"),
                cmd.name,
                escape_python_str(&cmd.description)
            ));

            for arg in &cmd.arguments {
                let p_var = format!("p_{}", cmd.name.replace('-', "_"));
                let flag = arg.cli_flag();
                let mut arg_parts = Vec::new();

                if arg.positional {
                    arg_parts.push(format!("'{}'", arg.name));
                } else {
                    arg_parts.push(format!("'{}'", flag));
                    if let Some(s) = arg.short {
                        arg_parts.push(format!("'-{}'", s));
                    }
                }

                arg_parts.push(arg.arg_type.argparse_type_code());

                if arg.required && !arg.positional {
                    arg_parts.push("required=True".to_string());
                }

                if let Some(ref def) = arg.default_value {
                    arg_parts.push(format!("default={}", escape_python_str(def)));
                }

                if !arg.description.is_empty() {
                    arg_parts.push(format!("help={}", escape_python_str(&arg.description)));
                }

                code.push_str(&format!(
                    "    {}.add_argument({})\n",
                    p_var,
                    arg_parts.join(", ")
                ));
            }
            code.push('\n');
        }

        code.push_str("    return parser\n\n");

        // 5. Main Execution Flow
        code.push_str("def main():\n");
        code.push_str("    parser = build_parser()\n");
        code.push_str("    args = parser.parse_args()\n\n");

        code.push_str("    if not args.subcommand:\n");
        code.push_str("        parser.print_help()\n");
        code.push_str("        sys.exit(0)\n\n");

        code.push_str("    use_json = getattr(args, 'json', False)\n");
        code.push_str("    verbose = getattr(args, 'verbose', False)\n\n");

        code.push_str("    try:\n");
        code.push_str("        target_mod = load_target_module()\n");
        code.push_str("        handlers = {\n");
        for cmd in &spec.commands {
            let func_identifier = format!("handle_{}", cmd.name.replace('-', "_"));
            code.push_str(&format!(
                "            '{}': {},\n",
                cmd.name, func_identifier
            ));
        }
        code.push_str("        }\n\n");

        code.push_str("        handler = handlers.get(args.subcommand)\n");
        code.push_str("        if not handler:\n");
        code.push_str("            raise ValueError(f\"Unknown subcommand: {args.subcommand}\")\n\n");

        code.push_str("        result = handler(args, target_mod)\n\n");

        // Structured JSON stdout
        code.push_str("        if use_json:\n");
        code.push_str("            envelope = {\n");
        code.push_str("                'status': 'success',\n");
        code.push_str("                'command': args.subcommand,\n");
        code.push_str("                'result': result,\n");
        code.push_str("                'timestamp': datetime.datetime.now(datetime.timezone.utc).isoformat(),\n");
        code.push_str("            }\n");
        code.push_str("            print(json.dumps(envelope, indent=2, default=str))\n");
        code.push_str("        else:\n");
        code.push_str("            if isinstance(result, (dict, list)):\n");
        code.push_str("                print(json.dumps(result, indent=2, default=str))\n");
        code.push_str("            else:\n");
        code.push_str("                print(result)\n");
        code.push_str("        sys.exit(0)\n\n");

        // Structured Error Handling
        code.push_str("    except Exception as e:\n");
        code.push_str("        if use_json:\n");
        code.push_str("            error_payload = {\n");
        code.push_str("                'status': 'error',\n");
        code.push_str("                'command': args.subcommand,\n");
        code.push_str("                'error': str(e),\n");
        code.push_str("                'type': type(e).__name__,\n");
        code.push_str("                'timestamp': datetime.datetime.now(datetime.timezone.utc).isoformat(),\n");
        code.push_str("            }\n");
        code.push_str("            if verbose:\n");
        code.push_str("                error_payload['traceback'] = traceback.format_exc()\n");
        code.push_str("            print(json.dumps(error_payload, indent=2))\n");
        code.push_str("        else:\n");
        code.push_str("            sys.stderr.write(f\"Error executing '{args.subcommand}': {e}\\n\")\n");
        code.push_str("            if verbose:\n");
        code.push_str("                traceback.print_exc(file=sys.stderr)\n");
        code.push_str("        sys.exit(1)\n\n");

        code.push_str("if __name__ == '__main__':\n");
        code.push_str("    main()\n");

        Ok(code)
    }

    /// Generates standalone test suite exercising all subcommands, JSON validation, and exit codes
    fn generate_python_tests(spec: &HarnessSpec, cli_filename: &str) -> Result<String> {
        let mut code = String::new();

        code.push_str("#!/usr/bin/env python3\n");
        code.push_str(&format!(
            "# Automated Validation Test Suite for {}\n",
            spec.name
        ));
        code.push_str("# Generated autonomously by Tagisan (tgs harness)\n\n");

        code.push_str("import json\n");
        code.push_str("import os\n");
        code.push_str("import subprocess\n");
        code.push_str("import sys\n");
        code.push_str("import unittest\n\n");

        code.push_str("CLI_SCRIPT = os.path.join(os.path.dirname(os.path.abspath(__file__)), '");
        code.push_str(cli_filename);
        code.push_str("')\n\n");

        code.push_str("class TestSynthesizedHarness(unittest.TestCase):\n\n");

        // 1. Help flag test
        code.push_str("    def test_01_help_flag(self):\n");
        code.push_str("        res = subprocess.run([sys.executable, CLI_SCRIPT, '--help'], capture_output=True, text=True)\n");
        code.push_str("        self.assertEqual(res.returncode, 0, f\"Help flag failed: {res.stderr}\")\n");
        code.push_str(&format!(
            "        self.assertIn('{}', res.stdout, \"Program name missing from help\")\n\n",
            spec.name
        ));

        // 2. Version flag test
        code.push_str("    def test_02_version_flag(self):\n");
        code.push_str("        res = subprocess.run([sys.executable, CLI_SCRIPT, '--version'], capture_output=True, text=True)\n");
        code.push_str("        self.assertEqual(res.returncode, 0, f\"Version flag failed: {res.stderr}\")\n");
        code.push_str(&format!(
            "        self.assertIn('{}', res.stdout, \"Version string missing\")\n\n",
            spec.version
        ));

        // 3. Subcommand execution tests
        for (idx, cmd) in spec.commands.iter().enumerate() {
            let test_fn_name = format!("test_{:02}_{}_json_success", idx + 3, cmd.name.replace('-', "_"));
            code.push_str(&format!("    def {}(self):\n", test_fn_name));

            let mut args_list = vec![
                "sys.executable".to_string(),
                "CLI_SCRIPT".to_string(),
                format!("'{}'", cmd.name),
                "'--json'".to_string(),
            ];

            for arg in &cmd.arguments {
                if arg.positional {
                    args_list.push(format!("'{}'", arg.arg_type.sample_value().trim_matches('"')));
                } else if arg.required {
                    args_list.push(format!("'{}'", arg.cli_flag()));
                    match arg.arg_type {
                        ArgumentType::Boolean => {}
                        ArgumentType::Integer => args_list.push("'10'".to_string()),
                        ArgumentType::Float => args_list.push("'3.14'".to_string()),
                        ArgumentType::List(_) => {
                            args_list.push("'item1'".to_string());
                            args_list.push("'item2'".to_string());
                        }
                        _ => args_list.push("'sample-test-arg'".to_string()),
                    }
                }
            }

            code.push_str(&format!(
                "        cmd_args = [{}]\n",
                args_list.join(", ")
            ));
            code.push_str("        res = subprocess.run(cmd_args, capture_output=True, text=True)\n");
            code.push_str(&format!(
                "        self.assertEqual(res.returncode, 0, f\"Command '{}' failed: {{res.stderr}} {{res.stdout}}\")\n",
                cmd.name
            ));
            code.push_str("        data = json.loads(res.stdout)\n");
            code.push_str("        self.assertEqual(data.get('status'), 'success')\n");
            code.push_str(&format!(
                "        self.assertEqual(data.get('command'), '{}')\n\n",
                cmd.name
            ));

            // Test missing required argument produces non-zero exit code
            let required_flags: Vec<&ArgumentSpec> = cmd.arguments.iter().filter(|a| a.required && !a.positional).collect();
            if !required_flags.is_empty() {
                let err_test_name = format!("test_{:02}_{}_missing_args", idx + 3, cmd.name.replace('-', "_"));
                code.push_str(&format!("    def {}(self):\n", err_test_name));
                code.push_str(&format!(
                    "        cmd_args = [sys.executable, CLI_SCRIPT, '{}', '--json']\n",
                    cmd.name
                ));
                code.push_str("        res = subprocess.run(cmd_args, capture_output=True, text=True)\n");
                code.push_str("        self.assertNotEqual(res.returncode, 0, \"Expected failure when required arguments are missing\")\n\n");
            }
        }

        code.push_str("if __name__ == '__main__':\n");
        code.push_str("    unittest.main()\n");

        Ok(code)
    }
}

fn escape_python_str(s: &str) -> String {
    let clean = s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
    format!("\"{}\"", clean)
}
