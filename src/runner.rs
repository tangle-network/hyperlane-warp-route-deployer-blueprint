use gadget_sdk::executor::process::manager::GadgetProcessManager;
use std::collections::HashMap;
use std::error::Error;

/// Function to run multiple commands and focus on the output of each command.
///
/// This function takes a GadgetProcessManager and a list of commands to run.
/// It runs each command using the manager and focuses on the output of each command.
/// The output of each command is stored in a HashMap with the command name as the key.
///
/// # Arguments
///
/// * `manager` - A mutable reference to the GadgetProcessManager used to run the commands.
/// * `commands` - A vector of tuples containing the command name and the command to run.
///
/// # Returns
///
/// Returns a Result containing a HashMap with the output of each command, or an error.
///
/// # Example
///
/// ```
/// use gadget_sdk::executor::process::manager::GadgetProcessManager;
/// use hyperlane_blueprint_template::runner::run_and_focus_multiple;
///
/// async fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let mut manager = GadgetProcessManager::new();
///     let commands = vec![
///         ("command1", "echo 'Hello World'"),
///         ("command2", "ls -l"),
///     ];
///     let outputs = run_and_focus_multiple(&mut manager, commands).await?;
///     Ok(())
/// }
/// ```
pub async fn run_and_focus_multiple<'a>(
    manager: &mut GadgetProcessManager,
    commands: Vec<(&'a str, &'a str)>,
) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let mut outputs = HashMap::new();
    for (name, command) in commands {
        let service = manager.run(name.to_string(), command).await?;
        let output = manager.focus_service_to_completion(service).await?;
        outputs.insert(name.to_string(), output);
    }
    Ok(outputs)
}
