use std::{fmt, str};
use curl::easy::Easy;
use serde_json::{Value};

const MOONRAKER_IP: &str = "192.168.1.17";

// #[derive(Debug)]
enum PrinterStatus {
    Standby,
    Printing(u8),
    Error(String),
    Pause(Option<String>),
}

// convert the status to string
// IMPORTANT: change the output here!!!
impl PrinterStatus {
    fn to_string(&self) -> String {
        match self {
            PrinterStatus::Standby => "%{B#111}Standby!".to_string(),
            PrinterStatus::Printing(progress) => format!("%{{B#0F0}}Printing: {}%", progress),
            PrinterStatus::Error(result) => format!("%{{B#F00}}ERROR!!! : {}", result),
            PrinterStatus::Pause(result_opt) => {
                match result_opt {
                    Some(result_real) => format!("%{{B#C00}}Pause! : {}", result_real),
                    None => "%{{B#C00}}Pause!".to_string()
                }
            }
        }
    }
}

// used for printing the values
impl fmt::Display for PrinterStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

fn bytes_to_str(bytes: &[u8]) -> &str {
    match str::from_utf8(bytes) {
        Ok(v) => v,
        Err(e) => panic!("error converting bytes to string! : {}", e),
    }
}

// ask moonraker for the state of the printer by the http api (/api/printer)
fn ask_moonraker() -> Result<String, ()> {
    let mut easy = Easy::new();
    let mut buffer: String = Default::default();
    easy.url(format!("http://{}/api/printer", MOONRAKER_IP).as_str()).unwrap();
    {
        let mut transfer = easy.transfer();
        transfer.write_function(|data| {
            buffer += bytes_to_str(&data);
            Ok(data.len())
        }).unwrap();
        transfer.perform().unwrap();
    }

    Ok(buffer)
}

fn get_status() -> Result<PrinterStatus, ()> {
    let text: String = ask_moonraker().unwrap();
    let json: Value = serde_json::from_str(text.as_str()).unwrap();
    
    for state in json["state"]["flags"].as_object().unwrap() {
        let (key, val) = state;
        if val == true {
            return match key.as_str() {
                "operational" | "ready" => Ok(PrinterStatus::Standby),
                "paused" | "pausing" | "cancelling" => Ok(PrinterStatus::Pause(None)),
                "printing" => Ok(PrinterStatus::Printing(0)),
                "error" | "closedOrError" => Ok(PrinterStatus::Error("".to_string())),
                _ => Err(())
            }
        }
    }
    Err(())
}

fn main() -> Result<(), ()> {
    let status = get_status()?;
    print!("{}", status);
    Ok(())
}
