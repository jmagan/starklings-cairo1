use crate::{
    clear_screen,
    exercise::{BbVerifyOptions, Exercise, Mode, State},
    utils,
};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::env;

// Verify that the provided container of Exercise objects
// can be compiled and run without any failures.
// Any such failures will be reported to the end user.
// If the Exercise being verified is a test, the verbose boolean
// determines whether or not the test harness outputs are displayed.
pub fn verify<'a>(
    exercises: impl IntoIterator<Item = &'a Exercise>,
    progress: (usize, usize),
) -> Result<(), &'a Exercise> {
    let (mut num_done, total) = progress;
    for exercise in exercises {
        clear_screen();
        let bar = ProgressBar::new(total as u64);
        bar.set_style(
            ProgressStyle::default_bar()
                .template("Progress: [{bar:60.green/red}] {pos}/{len} {msg}\n")
                .progress_chars("#>-"),
        );
        bar.set_position(num_done as u64);
        let exercise_result = {
            let run_result = match &exercise.mode {
                Mode::Build => utils::build_exercise(exercise),
                Mode::Execute(str) => utils::execute_exercise(exercise, str.clone()),
                Mode::BbProve(str) => utils::bb_prove_exercise(exercise, str.clone()),
                Mode::BbVerify(BbVerifyOptions {toml_file,save_files}) => utils::bb_prove_verify_exercise(exercise, toml_file.clone(), *save_files),
                Mode::Test => utils::test_exercise(exercise),
                _ => {
                    eprintln!("Invalid mode for exercise: {}", exercise.name);
                    return Err(exercise);
                }
            };
            match run_result {
                Ok(run_state) => Ok(prompt_for_completion(exercise, Some(run_state))),
                Err(_) => Err(()),
            }
        };
        if !exercise_result.unwrap_or(false) {
            return Err(exercise);
        }
        let percentage = num_done as f32 / total as f32 * 100.0;
        bar.set_message(format!("({percentage:.1} %)"));
        if exercise.looks_done(){
            num_done += 1;
        }
    }
    Ok(())
}

fn prompt_for_completion(exercise: &Exercise, prompt_output: Option<String>) -> bool {
    let context = match exercise.state() {
        State::Done => return true,
        State::Pending(context) => context,
    };

    if let Some(output) = prompt_output {
        utils::print_exercise_output(output);
    }

    utils::print_exercise_success(exercise);
    let no_emoji = env::var("NO_EMOJI").is_ok();

    let _clippy_success_msg = "The code is compiling, and Clippy is happy!";

    let success_msg = match exercise.mode {
        Mode::Build => "The code is compiling!",
        Mode::Execute(_) => "The code is compiling based on the witnesses!",
        Mode::BbProve(_) => "The code is compiling and a bb proof has been created!",
        Mode::Test => "The code is compiling, and the tests pass!",
        Mode::BbVerify(_) => "The code is compiling and the bb proof has been verified!",
        // Mode::Clippy => clippy_success_msg,
    };

    if no_emoji {
        println!("~*~ {success_msg} ~*~")
    } else {
        println!("🎉 🎉  {success_msg} 🎉 🎉")
    }
    println!();

    println!("You can keep working on this exercise,");
    println!(
        "or jump into the next one by removing the {} comment:",
        style("`I AM NOT DONE`").bold()
    );
    println!();
    for context_line in context {
        let formatted_line = if context_line.important {
            format!("{}", style(context_line.line).bold())
        } else {
            context_line.line.to_string()
        };

        println!(
            "{:>2} {}  {}",
            style(context_line.number).blue().bold(),
            style("|").blue(),
            formatted_line
        );
    }

    false
}
