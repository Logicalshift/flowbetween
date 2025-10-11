use crate::scenery::ui::*;

use flo_scene::*;
use flo_scene::programs::*;

use std::sync::*;

///
/// Generates a functikon for `expect_message` that expects a particular ToolState message
///
fn expect_toolstate(expected_state: ToolState) -> impl 'static + Fn(ToolState) -> Result<(), String> {
    move |actual_state| {
        if actual_state == expected_state {
            Ok(())
        } else {
            Err(format!("Expected toolstate {:?}, but got {:?}", expected_state, actual_state))
        }
    }
}

#[test]
pub fn add_tool_owner() {
    let scene = Scene::default();

    let tool_group  = ToolGroupId::new();
    let tool_type   = ToolTypeId::new();
    let tool_id_1   = ToolId::new();
    let tool_id_2   = ToolId::new();

    let test_program_id = SubProgramId::new();

    TestBuilder::new()
        .send_message(Tool::SetToolOwner(tool_type, test_program_id.into()))
        .send_message(Tool::CreateTool(tool_group, tool_type, tool_id_1))
        .send_message(Tool::CreateTool(tool_group, tool_type, tool_id_2))
        .expect_message(expect_toolstate(ToolState::AddTool(tool_id_1)))
        .expect_message(expect_toolstate(ToolState::AddTool(tool_id_2)))
        .run_in_scene(&scene, test_program_id);
}

#[test]
pub fn add_tool_to_location() {
    let scene = Scene::default();

    let tool_group  = ToolGroupId::new();
    let tool_type   = ToolTypeId::new();
    let tool_id_1   = ToolId::new();
    let tool_id_2   = ToolId::new();

    let test_program_id = SubProgramId::new();

    TestBuilder::new()
        .send_message(Tool::CreateTool(tool_group, tool_type, tool_id_1))
        .send_message(Tool::CreateTool(tool_group, tool_type, tool_id_2))
        .send_message(Tool::SetToolLocation(tool_id_1, test_program_id.into(), (0.0, 0.0)))
        .expect_message(expect_toolstate(ToolState::AddTool(tool_id_1)))
        .expect_message(expect_toolstate(ToolState::SetIcon(tool_id_1, Arc::new(vec![]))))
        .expect_message(expect_toolstate(ToolState::LocateTool(tool_id_1, (0.0, 0.0))))
        .run_in_scene(&scene, test_program_id);
}

#[test]
pub fn select_tool_location() {
    let scene = Scene::default();

    let tool_group  = ToolGroupId::new();
    let tool_type   = ToolTypeId::new();
    let tool_id_1   = ToolId::new();
    let tool_id_2   = ToolId::new();

    let test_program_id = SubProgramId::new();

    TestBuilder::new()
        .send_message(Tool::CreateTool(tool_group, tool_type, tool_id_1))
        .send_message(Tool::CreateTool(tool_group, tool_type, tool_id_2))
        .send_message(Tool::SetToolLocation(tool_id_1, test_program_id.into(), (0.0, 0.0)))
        .expect_message(expect_toolstate(ToolState::AddTool(tool_id_1)))
        .expect_message(expect_toolstate(ToolState::SetIcon(tool_id_1, Arc::new(vec![]))))
        .expect_message(expect_toolstate(ToolState::LocateTool(tool_id_1, (0.0, 0.0))))
        .send_message(Tool::Select(tool_id_1))
        .expect_message(expect_toolstate(ToolState::Select(tool_id_1)))
        .send_message(Tool::Select(tool_id_2))
        .expect_message(expect_toolstate(ToolState::Deselect(tool_id_1)))
        .send_message(Tool::Select(tool_id_1))
        .expect_message(expect_toolstate(ToolState::Select(tool_id_1)))
        .run_in_scene(&scene, test_program_id);
}

#[test]
pub fn select_tool_owner() {
    let scene = Scene::default();

    let tool_group  = ToolGroupId::new();
    let tool_type   = ToolTypeId::new();
    let tool_id_1   = ToolId::new();
    let tool_id_2   = ToolId::new();

    let test_program_id = SubProgramId::new();

    TestBuilder::new()
        .send_message(Tool::SetToolOwner(tool_type, test_program_id.into()))
        .send_message(Tool::CreateTool(tool_group, tool_type, tool_id_1))
        .send_message(Tool::CreateTool(tool_group, tool_type, tool_id_2))
        .expect_message(expect_toolstate(ToolState::AddTool(tool_id_1)))
        .expect_message(expect_toolstate(ToolState::AddTool(tool_id_2)))
        .send_message(Tool::Select(tool_id_1))
        .expect_message(expect_toolstate(ToolState::Select(tool_id_1)))
        .send_message(Tool::Select(tool_id_2))
        .expect_message(expect_toolstate(ToolState::Deselect(tool_id_1)))
        .expect_message(expect_toolstate(ToolState::Select(tool_id_2)))
        .send_message(Tool::Select(tool_id_1))
        .expect_message(expect_toolstate(ToolState::Deselect(tool_id_2)))
        .expect_message(expect_toolstate(ToolState::Select(tool_id_1)))
        .run_in_scene(&scene, test_program_id);
}

#[test]
pub fn does_not_reselect_same_tool() {
    let scene = Scene::default();

    let tool_group  = ToolGroupId::new();
    let tool_type   = ToolTypeId::new();
    let tool_id_1   = ToolId::new();
    let tool_id_2   = ToolId::new();

    let test_program_id = SubProgramId::new();

    println!("ToolID1 = {:?}", tool_id_1);
    println!("ToolID2 = {:?}", tool_id_2);

    TestBuilder::new()
        .send_message(Tool::SetToolOwner(tool_type, test_program_id.into()))
        .send_message(Tool::CreateTool(tool_group, tool_type, tool_id_1))
        .send_message(Tool::CreateTool(tool_group, tool_type, tool_id_2))
        .expect_message(expect_toolstate(ToolState::AddTool(tool_id_1)))
        .expect_message(expect_toolstate(ToolState::AddTool(tool_id_2)))
        .send_message(Tool::Select(tool_id_1))
        .send_message(Tool::Select(tool_id_1))
        .expect_message(expect_toolstate(ToolState::Select(tool_id_1)))
        .send_message(Tool::Select(tool_id_2))
        .expect_message(expect_toolstate(ToolState::Deselect(tool_id_1)))
        .expect_message(expect_toolstate(ToolState::Select(tool_id_2)))
        .run_in_scene(&scene, test_program_id);
}

#[test]
pub fn select_tools_in_groups_owner() {
    let scene = Scene::default();

    let tool_group_1        = ToolGroupId::new();
    let tool_group_2        = ToolGroupId::new();
    let tool_type           = ToolTypeId::new();
    let tool_id_1_group1    = ToolId::new();
    let tool_id_2_group1    = ToolId::new();
    let tool_id_1_group2    = ToolId::new();
    let tool_id_2_group2    = ToolId::new();

    let test_program_id = SubProgramId::new();

    TestBuilder::new()
        .send_message(Tool::SetToolOwner(tool_type, test_program_id.into()))
        .send_message(Tool::CreateTool(tool_group_1, tool_type, tool_id_1_group1))
        .send_message(Tool::CreateTool(tool_group_1, tool_type, tool_id_2_group1))
        .send_message(Tool::CreateTool(tool_group_2, tool_type, tool_id_1_group2))
        .send_message(Tool::CreateTool(tool_group_2, tool_type, tool_id_2_group2))
        .expect_message(expect_toolstate(ToolState::AddTool(tool_id_1_group1)))
        .expect_message(expect_toolstate(ToolState::AddTool(tool_id_2_group1)))
        .expect_message(expect_toolstate(ToolState::AddTool(tool_id_1_group2)))
        .expect_message(expect_toolstate(ToolState::AddTool(tool_id_2_group2)))
        .send_message(Tool::Select(tool_id_1_group1))
        .send_message(Tool::Select(tool_id_1_group2))
        .expect_message(expect_toolstate(ToolState::Select(tool_id_1_group1)))
        .expect_message(expect_toolstate(ToolState::Select(tool_id_1_group2)))
        .send_message(Tool::Select(tool_id_2_group1))
        .expect_message(expect_toolstate(ToolState::Deselect(tool_id_1_group1)))
        .expect_message(expect_toolstate(ToolState::Select(tool_id_2_group1)))
        .send_message(Tool::Select(tool_id_2_group2))
        .expect_message(expect_toolstate(ToolState::Deselect(tool_id_1_group2)))
        .expect_message(expect_toolstate(ToolState::Select(tool_id_2_group2)))
        .send_message(Tool::Select(tool_id_1_group1))
        .expect_message(expect_toolstate(ToolState::Deselect(tool_id_2_group1)))
        .expect_message(expect_toolstate(ToolState::Select(tool_id_1_group1)))
        .run_in_scene(&scene, test_program_id);
}
