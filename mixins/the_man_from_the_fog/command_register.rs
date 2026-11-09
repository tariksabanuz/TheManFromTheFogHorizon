// `/test`: adds the command to the server's command tree so chat completes it.
fn the_man_from_the_fog_register_test(commands: &mut Vec<horizon_client::mod_api::NativeCommand>) {
    commands.push(horizon_client::mod_api::NativeCommand::new("test"));
}
