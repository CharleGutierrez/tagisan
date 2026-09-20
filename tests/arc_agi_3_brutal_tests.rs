use std::time::Duration;
use tagisan::engine::arc_agi::*;
use tagisan::tools::ToolHandler;

#[test]
fn test_arc_colors_and_palette() {
    assert_eq!(ArcColor::ALL.len(), 10);
    assert_eq!(ArcColor::name(0), "Black");
    assert_eq!(ArcColor::name(1), "Blue");
    assert_eq!(ArcColor::name(2), "Red");
    assert_eq!(ArcColor::name(3), "Green");
    assert_eq!(ArcColor::name(4), "Yellow");
    assert_eq!(ArcColor::name(5), "Gray");
    assert_eq!(ArcColor::name(6), "Magenta");
    assert_eq!(ArcColor::name(7), "Orange");
    assert_eq!(ArcColor::name(8), "Azure");
    assert_eq!(ArcColor::name(9), "Maroon");

    assert_eq!(ArcColor::hex(0), "#000000");
    assert_eq!(ArcColor::hex(1), "#0074D9");
    assert_eq!(ArcColor::hex(2), "#FF4136");
    assert_eq!(ArcColor::hex(8), "#7FDBFF");

    assert_eq!(ArcColor::from_u8(0), Some(ArcColor::Black));
    assert_eq!(ArcColor::from_u8(9), Some(ArcColor::Maroon));
    assert_eq!(ArcColor::from_u8(10), None);
}

#[test]
fn test_arc_grid_creation_and_ascii() {
    let mut grid = ArcGrid::new(3, 4);
    assert_eq!(grid.dims(), (3, 4));
    assert_eq!(grid.height(), 3);
    assert_eq!(grid.width(), 4);

    assert!(grid.set(0, 0, 1));
    assert!(grid.set(1, 2, 2));
    assert!(grid.set(2, 3, 3));
    assert!(!grid.set(5, 5, 1)); // out of bounds

    assert_eq!(grid.get(0, 0), Some(1));
    assert_eq!(grid.get(1, 2), Some(2));
    assert_eq!(grid.get(2, 3), Some(3));
    assert_eq!(grid.get(0, 1), Some(0));
    assert_eq!(grid.get(9, 9), None);

    let ascii = grid.to_ascii();
    let parsed = ArcGrid::from_ascii(&ascii).expect("Failed to parse ascii grid");
    assert_eq!(grid, parsed);
}

#[test]
fn test_geometric_rotations_and_reflections() {
    // 2x3 grid:
    // 1 2 3
    // 4 5 6
    let grid = ArcGrid::from_slice(&[&[1, 2, 3], &[4, 5, 6]]);

    // Rotate 90:
    // 4 1
    // 5 2
    // 6 3
    let r90 = grid.rotate90();
    assert_eq!(r90.dims(), (3, 2));
    assert_eq!(r90.cells, vec![vec![4, 1], vec![5, 2], vec![6, 3]]);

    // Rotate 180:
    // 6 5 4
    // 3 2 1
    let r180 = grid.rotate180();
    assert_eq!(r180.dims(), (2, 3));
    assert_eq!(r180.cells, vec![vec![6, 5, 4], vec![3, 2, 1]]);

    // Rotate 270:
    // 3 6
    // 2 5
    // 1 4
    let r270 = grid.rotate270();
    assert_eq!(r270.dims(), (3, 2));
    assert_eq!(r270.cells, vec![vec![3, 6], vec![2, 5], vec![1, 4]]);

    // Rotate 360 equals original
    assert_eq!(r90.rotate90().rotate90().rotate90(), grid);

    // Reflect Horizontal (flip rows):
    // 4 5 6
    // 1 2 3
    let ref_h = grid.reflect_horizontal();
    assert_eq!(ref_h.cells, vec![vec![4, 5, 6], vec![1, 2, 3]]);

    // Reflect Vertical (flip columns):
    // 3 2 1
    // 6 5 4
    let ref_v = grid.reflect_vertical();
    assert_eq!(ref_v.cells, vec![vec![3, 2, 1], vec![6, 5, 4]]);

    // Transpose (Diagonal Main):
    // 1 4
    // 2 5
    // 3 6
    let ref_diag = grid.reflect_diagonal_main();
    assert_eq!(ref_diag.cells, vec![vec![1, 4], vec![2, 5], vec![3, 6]]);
}

#[test]
fn test_grid_transformations_crop_pad_zoom_tile() {
    let grid = ArcGrid::from_slice(&[&[0, 1], &[2, 0]]);

    // Zoom factor 2 -> 4x4
    let zoomed = grid.zoom(2);
    assert_eq!(zoomed.dims(), (4, 4));
    assert_eq!(
        zoomed.cells,
        vec![
            vec![0, 0, 1, 1],
            vec![0, 0, 1, 1],
            vec![2, 2, 0, 0],
            vec![2, 2, 0, 0]
        ]
    );

    // Tile 2x3 -> 4x6
    let tiled = grid.tile(2, 3);
    assert_eq!(tiled.dims(), (4, 6));
    assert_eq!(tiled.get(0, 0), Some(0));
    assert_eq!(tiled.get(0, 1), Some(1));
    assert_eq!(tiled.get(0, 2), Some(0));
    assert_eq!(tiled.get(0, 3), Some(1));

    // Pad
    let padded = grid.pad(1, 1, 2, 2, 9);
    assert_eq!(padded.dims(), (4, 6));
    assert_eq!(padded.get(0, 0), Some(9));
    assert_eq!(padded.get(1, 2), Some(0));
    assert_eq!(padded.get(1, 3), Some(1));

    // Crop to content
    let bbox = padded.content_bounding_box(9).unwrap();
    let cropped = padded.crop(&bbox);
    assert_eq!(cropped, grid);
}

#[test]
fn test_symmetry_detection() {
    // Vertically symmetric grid:
    // 1 2 1
    // 3 4 3
    let v_sym = ArcGrid::from_slice(&[&[1, 2, 1], &[3, 4, 3]]);
    let rep = v_sym.detect_symmetries();
    assert!(rep.vertical);
    assert!(!rep.horizontal);

    // Horizontally symmetric grid:
    // 1 2
    // 1 2
    let h_sym = ArcGrid::from_slice(&[&[1, 2], &[1, 2]]);
    let rep_h = h_sym.detect_symmetries();
    assert!(rep_h.horizontal);
    assert!(!rep_h.vertical);

    // Fully symmetric square:
    // 1 1
    // 1 1
    let full_sym = ArcGrid::from_slice(&[&[1, 1], &[1, 1]]);
    let rep_full = full_sym.detect_symmetries();
    assert!(rep_full.horizontal);
    assert!(rep_full.vertical);
    assert!(rep_full.diagonal_main);
    assert!(rep_full.rotational_90);
    assert!(rep_full.rotational_180);
}

#[test]
fn test_gravity_simulation() {
    // 4x3 grid with non-zero pixels floating:
    // . 1 .
    // . . 2
    // . 3 .
    // . . .
    let grid = ArcGrid::from_slice(&[
        &[0, 1, 0],
        &[0, 0, 2],
        &[0, 3, 0],
        &[0, 0, 0],
    ]);

    // Simulate Gravity Down:
    // . . .
    // . . .
    // . 1 .
    // . 3 2
    let down = grid.simulate_gravity(GravityDirection::Down, 0);
    assert_eq!(
        down.cells,
        vec![
            vec![0, 0, 0],
            vec![0, 0, 0],
            vec![0, 1, 0],
            vec![0, 3, 2],
        ]
    );

    // Simulate Gravity Up:
    // . 1 2
    // . 3 .
    // . . .
    // . . .
    let up = grid.simulate_gravity(GravityDirection::Up, 0);
    assert_eq!(
        up.cells,
        vec![
            vec![0, 1, 2],
            vec![0, 3, 0],
            vec![0, 0, 0],
            vec![0, 0, 0],
        ]
    );

    // Simulate Gravity Right:
    let right = grid.simulate_gravity(GravityDirection::Right, 0);
    assert_eq!(
        right.cells,
        vec![
            vec![0, 0, 1],
            vec![0, 0, 2],
            vec![0, 0, 3],
            vec![0, 0, 0],
        ]
    );
}

#[test]
fn test_object_segmentation() {
    // 4x4 grid with two disjoint objects:
    // 1 1 . 2
    // 1 . . 2
    // . . . 2
    // 3 3 . .
    let grid = ArcGrid::from_slice(&[
        &[1, 1, 0, 2],
        &[1, 0, 0, 2],
        &[0, 0, 0, 2],
        &[3, 3, 0, 0],
    ]);

    let objects = grid.find_objects(Connectivity::FourWay, Some(0), true);
    assert_eq!(objects.len(), 3);

    // Find object of color 1
    let obj1 = objects.iter().find(|o| o.color == Some(1)).unwrap();
    assert_eq!(obj1.area(), 3);
    assert_eq!(obj1.bbox, BoundingBox::new(0, 0, 1, 1));

    // Find object of color 2
    let obj2 = objects.iter().find(|o| o.color == Some(2)).unwrap();
    assert_eq!(obj2.area(), 3);
    assert_eq!(obj2.bbox, BoundingBox::new(0, 3, 2, 3));

    // Find object of color 3
    let obj3 = objects.iter().find(|o| o.color == Some(3)).unwrap();
    assert_eq!(obj3.area(), 2);
    assert_eq!(obj3.bbox, BoundingBox::new(3, 0, 3, 1));
}

#[test]
fn test_dsl_program_execution_and_parsing() {
    // Rotate(90) | FillColor(1, 5)
    let code = "Rotate(90) | FillColor(1, 5)";
    let prog = DslProgram::from_code(code).expect("Failed to parse DSL program");
    assert_eq!(prog.len(), 2);
    assert_eq!(prog.to_code(), "Rotate(90) | FillColor(1, 5)");

    let input = ArcGrid::from_slice(&[&[1, 0], &[0, 2]]);
    let output = prog.execute(&input).expect("Execution failed");

    // Rotate 90:
    // 0 1
    // 2 0
    // FillColor(1, 5):
    // 0 5
    // 2 0
    assert_eq!(output.cells, vec![vec![0, 5], vec![2, 0]]);
}

#[test]
fn test_dsl_timeout_and_step_limits() {
    let mut prog = DslProgram::new().with_step_limit(3);
    prog.push(DslOp::Rotate(RotationAngle::Deg90));
    prog.push(DslOp::Rotate(RotationAngle::Deg90));
    prog.push(DslOp::Rotate(RotationAngle::Deg90));
    prog.push(DslOp::Rotate(RotationAngle::Deg90)); // 4 ops > 3 limit

    let input = ArcGrid::new(2, 2);
    let res = prog.execute(&input);
    assert!(matches!(res, Err(DslError::StepLimitExceeded(3))));
}

#[test]
fn test_mcts_search_color_swap() {
    // Pattern: Replace color 1 with color 4
    let in1 = ArcGrid::from_slice(&[&[1, 0], &[0, 1]]);
    let out1 = ArcGrid::from_slice(&[&[4, 0], &[0, 4]]);
    let in2 = ArcGrid::from_slice(&[&[1, 1], &[0, 0]]);
    let out2 = ArcGrid::from_slice(&[&[4, 4], &[0, 0]]);

    let pairs = [(&in1, &out1), (&in2, &out2)];
    let invariants = ArcTaskInvariants::deduce(&pairs);

    let config = MctsConfig {
        max_iterations: 500,
        max_depth: 2,
        exploration_c: 1.414,
        timeout: Duration::from_secs(2),
        max_program_length: 2,
    };

    let mut searcher = MctsSearcher::new(config);
    let (prog, candidates) = searcher.search(&pairs, Some(&invariants));

    assert!(prog.is_some(), "MCTS should discover color swap");
    let p = prog.unwrap();
    assert_eq!(p.execute(&in1).unwrap(), out1);
    assert_eq!(p.execute(&in2).unwrap(), out2);
    assert!(!candidates.is_empty());
}

#[test]
fn test_mcts_search_gravity() {
    // Pattern: Gravity Down
    let in1 = ArcGrid::from_slice(&[&[1, 0], &[0, 0]]);
    let out1 = ArcGrid::from_slice(&[&[0, 0], &[1, 0]]);
    let in2 = ArcGrid::from_slice(&[&[0, 2], &[0, 0]]);
    let out2 = ArcGrid::from_slice(&[&[0, 0], &[0, 2]]);

    let pairs = [(&in1, &out1), (&in2, &out2)];
    let invariants = ArcTaskInvariants::deduce(&pairs);

    let config = MctsConfig {
        max_iterations: 500,
        max_depth: 2,
        exploration_c: 1.414,
        timeout: Duration::from_secs(2),
        max_program_length: 2,
    };

    let mut searcher = MctsSearcher::new(config);
    let (prog, _) = searcher.search(&pairs, Some(&invariants));

    assert!(prog.is_some(), "MCTS should discover Gravity(Down)");
    let p = prog.unwrap();
    assert_eq!(p.execute(&in1).unwrap(), out1);
    assert_eq!(p.execute(&in2).unwrap(), out2);
}

#[test]
fn test_mcts_search_crop_bounding_box() {
    // Pattern: Crop to content
    let in1 = ArcGrid::from_slice(&[
        &[0, 0, 0],
        &[0, 2, 0],
        &[0, 0, 0],
    ]);
    let out1 = ArcGrid::from_slice(&[&[2]]);

    let in2 = ArcGrid::from_slice(&[
        &[0, 0, 0],
        &[0, 3, 3],
        &[0, 0, 0],
    ]);
    let out2 = ArcGrid::from_slice(&[&[3, 3]]);

    let pairs = [(&in1, &out1), (&in2, &out2)];
    let invariants = ArcTaskInvariants::deduce(&pairs);

    let config = MctsConfig {
        max_iterations: 500,
        max_depth: 2,
        exploration_c: 1.414,
        timeout: Duration::from_secs(2),
        max_program_length: 2,
    };

    let mut searcher = MctsSearcher::new(config);
    let (prog, _) = searcher.search(&pairs, Some(&invariants));

    assert!(prog.is_some(), "MCTS should discover CropBoundingBox");
    let p = prog.unwrap();
    assert_eq!(p.execute(&in1).unwrap(), out1);
    assert_eq!(p.execute(&in2).unwrap(), out2);
}

#[test]
fn test_master_solver_end_to_end() {
    let in1 = ArcGrid::from_slice(&[&[1, 2], &[3, 4]]);
    let out1 = in1.rotate90();

    let in2 = ArcGrid::from_slice(&[&[5, 6], &[7, 8]]);
    let out2 = in2.rotate90();

    let test_in = ArcGrid::from_slice(&[&[2, 3], &[4, 5]]);
    let expected_test_out = test_in.rotate90();

    let task = ArcTask::new(
        vec![
            ArcExample::new(in1, Some(out1)),
            ArcExample::new(in2, Some(out2)),
        ],
        vec![ArcExample::new(test_in, None)],
    );

    let solver = ArcAgi3Solver::new().with_max_time(Duration::from_secs(5));
    let result = solver.solve(&task).expect("Solver failed");

    assert!(result.solved_training);
    assert_eq!(result.predictions.len(), 1);
    assert_eq!(result.predictions[0].candidate_1, expected_test_out);
    assert_eq!(result.predictions[0].confidence, 100);
}

#[tokio::test]
async fn test_arc_agi_solve_tool_handler() {
    let in1 = vec![vec![1, 0], vec![0, 1]];
    let out1 = vec![vec![0, 1], vec![1, 0]]; // Reflect(Vertical) or Rotate(90)+...

    let task_json = serde_json::json!({
        "train": [
            { "input": in1, "output": out1 }
        ],
        "test": [
            { "input": [[2, 0], [0, 2]] }
        ]
    });

    let tool = ArcAgiSolveTool::new();
    let res = tool
        .execute(serde_json::json!({
            "task": task_json,
            "max_time_secs": 5,
            "strategy": "auto"
        }))
        .await
        .expect("Tool execution failed");

    let val: serde_json::Value = serde_json::from_str(&res).expect("Invalid JSON returned by tool");
    assert_eq!(val["status"], "success");
    assert_eq!(val["solved_training"], true);
    assert!(val["predictions"].as_array().unwrap().len() == 1);
}

#[test]
fn test_neurosymbolic_sandbox_diffing() {
    let in1 = ArcGrid::from_slice(&[&[1, 2], &[3, 4]]);
    let target = ArcGrid::from_slice(&[&[1, 2], &[3, 9]]); // mismatch at (1, 1)

    let prog = DslProgram::new(); // Identity
    let pairs = [(&in1, &target)];

    let report = NeurosymbolicEngine::verify_in_sandbox(&prog, &pairs);
    assert!(!report.all_passed);
    assert_eq!(report.pass_count, 0);
    assert_eq!(report.pair_results[0].error_count, 1);
    assert_eq!(report.pair_results[0].pixel_errors[0].row, 1);
    assert_eq!(report.pair_results[0].pixel_errors[0].col, 1);
    assert_eq!(report.pair_results[0].pixel_errors[0].expected, 9);
    assert_eq!(report.pair_results[0].pixel_errors[0].actual, 4);

    let prompt = NeurosymbolicEngine::generate_refinement_prompt(&report);
    assert!(prompt.contains("FAILED 1 of 1"));
    assert!(prompt.contains("At (1, 1)"));
}
