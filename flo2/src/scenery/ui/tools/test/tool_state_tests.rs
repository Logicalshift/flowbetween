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
        .send_message(Tool::SetToolTypeOwner(tool_type, test_program_id.into()))
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
        .send_message(Tool::SetToolTypeOwner(tool_type, test_program_id.into()))
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
        .send_message(Tool::SetToolTypeOwner(tool_type, test_program_id.into()))
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
        .send_message(Tool::SetToolTypeOwner(tool_type, test_program_id.into()))
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

#[test]
pub fn select_tools_as_group_when_joined() {
    let scene = Scene::default();

    // All of these tools will be of the same 'type' (it's pretty rare that tools of a common type will be in different groups in reality, but convenient for this test)
    let tool_type = ToolTypeId::new();

    // Three tool groups (we'll put our selections in all of them)
    let tool_group_1 = ToolGroupId::new();
    let tool_group_2 = ToolGroupId::new();
    let tool_group_3 = ToolGroupId::new();

    // One tool per group that's 'independent'
    let independent_1 = ToolId::new();
    let independent_2 = ToolId::new();
    let independent_3 = ToolId::new();

    // One tool per group that's joined
    let joined_1 = ToolId::new();
    let joined_2 = ToolId::new();
    let joined_3 = ToolId::new();

    // The test progrm ID acts as the tool's owner
    let test_program_id = SubProgramId::new();

    TestBuilder::new()
        .send_message(Tool::SetToolTypeOwner(tool_type, test_program_id.into()))

        // Create our six tools
        .send_message(Tool::CreateTool(tool_group_1, tool_type, independent_1))
        .send_message(Tool::CreateTool(tool_group_2, tool_type, independent_2))
        .send_message(Tool::CreateTool(tool_group_3, tool_type, independent_3))
        .send_message(Tool::CreateTool(tool_group_1, tool_type, joined_1))
        .send_message(Tool::CreateTool(tool_group_2, tool_type, joined_2))
        .send_message(Tool::CreateTool(tool_group_3, tool_type, joined_3))

        .expect_message(expect_toolstate(ToolState::AddTool(independent_1)))
        .expect_message(expect_toolstate(ToolState::AddTool(independent_2)))
        .expect_message(expect_toolstate(ToolState::AddTool(independent_3)))
        .expect_message(expect_toolstate(ToolState::AddTool(joined_1)))
        .expect_message(expect_toolstate(ToolState::AddTool(joined_2)))
        .expect_message(expect_toolstate(ToolState::AddTool(joined_3)))

        // Select the independent tools
        .send_message(Tool::Select(independent_1))
        .send_message(Tool::Select(independent_2))
        .send_message(Tool::Select(independent_3))

        .expect_message(expect_toolstate(ToolState::Select(independent_1)))
        .expect_message(expect_toolstate(ToolState::Select(independent_2)))
        .expect_message(expect_toolstate(ToolState::Select(independent_3)))

        // Join up the joined tools
        .send_message(Tool::JoinTools(joined_1, joined_2))
        // .send_message(Tool::JoinTools(joined_1, joined_3))

        // Selecting the joined tool should select the other two tools
        .send_message(Tool::Select(joined_1))

        .expect_message(expect_toolstate(ToolState::Deselect(independent_1)))
        .expect_message(expect_toolstate(ToolState::Deselect(independent_2)))
        // .expect_message(expect_toolstate(ToolState::Deselect(independent_3)))
        .expect_message(expect_toolstate(ToolState::Select(joined_1)))
        .expect_message(expect_toolstate(ToolState::Select(joined_2)))
        // .expect_message(expect_toolstate(ToolState::Select(joined_3)))

        .run_in_scene(&scene, test_program_id);
}

#[test]
pub fn secondary_tool_selection_restores_after_joined_tool_deselected() {

}

#[test]
pub fn change_secondary_tool_selection_after_group_select() {

}

#[test]
pub fn changed_secondary_tool_not_restored_after_group_deselected() {

}