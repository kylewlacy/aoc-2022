use std::io::BufRead;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = std::io::stdin().lock();

    let mut current_most = 0;
    let mut best_most = 0;
    for line in stdin.lines() {
        let line = line?;

        if line.is_empty() {
            if current_most > best_most {
                best_most = current_most;
            }

            current_most = 0;
        } else {
            let calories: u64 = line.parse()?;
            current_most += calories;
        }
    }

    if current_most > best_most {
        best_most = current_most;
    }

    println!("{}", best_most);

    Ok(())
}
