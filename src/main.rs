use std::io::{stdin, stdout, Write};
use std::fs::OpenOptions;
use std::time::Duration;
use std::str::from_utf8;
use std::error::Error;
use std::path::Path;
use std::env;
use std::io;

use plotpy::{Curve, Plot};

use csv::Writer;

use serialport;

use chrono;

#[derive(PartialEq)]
enum LastItem {
    Temperature,
    Humidity,
    Pressure,
}

struct DataItem {
    temperature: f32,
    humidity: f32,
    pressure: f32,
    last: LastItem,
}

impl DataItem {
    fn new() -> DataItem {
        DataItem {
            temperature: 0.0,
            humidity: 0.0,
            pressure: 0.0,
            last: LastItem::Temperature,
        }
    }
}

fn parse_result(serial_buf: &Vec<u8>, t: usize, mut data: DataItem, path: &str) -> DataItem {
    let buf = from_utf8(&serial_buf[..t]).unwrap();
    
    let file = OpenOptions::new()
                .write(true)
                .append(true)
                .open(path)
                .unwrap();

    let mut wtr = Writer::from_writer(file);

    match buf.trim().chars().nth(0) {
        Some(d) => {
            match d {
                'T' => {
                    let serialout: Vec<&str> = buf.split(" ").collect();

                    match data.last {
                        LastItem::Pressure => {
                            println!("");
                            data.temperature = serialout[1].trim().parse::<f32>().unwrap();
                        },
                        _ => (),
                    }

                    println!("Temperature is {}", serialout[1]);

                    data.last = LastItem::Temperature;
                },
                'H' => {
                    let serialout: Vec<&str> = buf.split(" ").collect();

                    match data.last {
                        LastItem::Temperature => {
                            data.humidity = serialout[1].trim().parse::<f32>().unwrap();
                        },
                        _ => (),
                    }

                    println!("Humidity is {}", serialout[1]);

                    data.last = LastItem::Humidity;
                },
                'P' => {
                    let serialout: Vec<&str> = buf.split(" ").collect();

                    match data.last {
                        LastItem::Humidity => {
                            data.pressure = serialout[1].trim().parse::<f32>().unwrap();
                        },
                        _ => (),
                    }

                    println!("Pressure is {}", serialout[1]);

                    if data.temperature != 0.0 && data.humidity != 0.0 && data.pressure != 0.0 {
                        let time: String = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                        let temp: String = data.temperature.to_string();
                        let humi: String = data.humidity.to_string();
                        let pres: String = data.pressure.to_string();

                        println!("Data recorded at {}", time);

                        match wtr.write_record(&[time, temp, humi, pres]){
                            Ok(()) => (),
                            Err(e) => println!("{}", e),
                        }
                    
                        match wtr.flush(){
                            Ok(()) => (),
                            Err(e) => println!("{}", e),
                        }

                        data = DataItem::new();
                    }

                    data.last = LastItem::Pressure;
                },
                _ => (),
            }
        }
        None => (),
    }

    data
}

fn write_header(path: &str) -> Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_path(path)?;

    wtr.write_record(&["Time", "Temperature", "Humidity", "Pressure"])?;
    wtr.flush()?;

    Ok(())

}

fn getdata(port_name: &str, baud_rate: u32, path: &str) {
    let mut data = DataItem::new();

    let port = serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(10))
        .open();

    match port {
        Ok(mut port) => {
            let mut serial_buf: Vec<u8> = vec![0; 1000];
            println!("Receiving data on {} at {} baud:", &port_name, &baud_rate);
            loop {
                match port.read(serial_buf.as_mut_slice()) {
                    Ok(t) => {
                        data = parse_result(&serial_buf, t, data, path);
                    },
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                    Err(e) => eprintln!("{:?}", e),
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to open \"{}\". Error: {}", port_name, e);
            ::std::process::exit(1);
        }
    }
}

fn createplot() -> Result<(), Box<dyn Error>>{
    let mut rdr = csv::ReaderBuilder::new()
                                      .delimiter(b';')
                                      .from_path("data.csv")
                                      .unwrap();

    let mut x: Vec<f32> = Vec::new();
    let mut temp_y: Vec<f32> = Vec::new();
    let mut humi_y: Vec<f32> = Vec::new();
    let mut pres_y: Vec<f32> = Vec::new();

    let mut time: f32 = 2.0;

    for result in rdr.records() {
        let record = result?;

        x.push(time.clone());
        time += 3.0;

        match record.get(1) {
            Some(i) => temp_y.push(i.to_string().parse::<f32>().unwrap().clone()),
            _ => (),
        }

        match record.get(2) {
            Some(i) => humi_y.push(i.to_string().parse::<f32>().unwrap().clone()),
            _ => (),
        }

        match record.get(3) {
            Some(i) => pres_y.push(i.to_string().parse::<f32>().unwrap().clone()),
            _ => (),
        }
    }

    // Temperature
    let mut temp_curve = Curve::new();
    temp_curve.draw(&x, &temp_y);

    let mut temp_plot = Plot::new();
    temp_plot.set_title("Temperature");
    temp_plot.add(&temp_curve);
    temp_plot.set_labels("Temps (en heures)", "Température (en °C)");

    temp_plot.save(Path::new("temperature.svg"))?;

    // Humidity
    let mut humi_curve = Curve::new();
    humi_curve.draw(&x, &humi_y);

    let mut humi_plot = Plot::new();
    humi_plot.set_title("Humidity");
    humi_plot.add(&humi_curve);
    humi_plot.set_labels("Temps (en heures)", "Humidité (en %)");

    humi_plot.save(Path::new("humidity.svg"))?;

    // Pressure
    let mut pres_curve = Curve::new();
    pres_curve.draw(&x, &pres_y);

    let mut pres_plot = Plot::new();
    pres_plot.set_title("Pressure");
    pres_plot.add(&pres_curve);
    pres_plot.set_labels("Temps (en heures)", "Pression (en hPa)");

    pres_plot.save(Path::new("pressure.svg"))?;

    Ok(())
}

fn main() {    
    let port_name: &str = "/dev/ttyACM0";
    let baud_rate: u32 = 9600;
    let path: &str = "data.csv";
    //let args: Vec<String> = env::args().collect();

    if env::args().len() < 2 {
        loop {
            let mut input = String::new();
            print!("Please enter a command > ");
            
            let _ = stdout().flush();
            stdin().read_line(&mut input).expect("Did not enter a correct string");
            
            let command = input.trim();
            
            match command {
                "help" => {
                    println!("  help              show this menu");
                    println!("  writecsv          recreate csv and write headers");
                    println!("  writedata         collect data from arduino");
                    println!("  createplot        create matplotlib graph from csv");
                    println!("  exit              close the application");
                },
                "writecsv" => {
                    if let Err(e) = write_header(path) {
                        eprintln!("{}", e)
                    }
                },
                "writedata" => getdata(port_name, baud_rate, path),
                "createplot" => {
                    if let Err(e) = createplot() {
                        eprintln!("{}", e)
                    }
                },
                "exit" => return,
                _ => println!("Error"),
            }
        }
    }
        
    if !(Path::new(path).exists()) {
        println!("csv file don't exists, creating one...");

        if let Err(e) = write_header(path) {
            eprintln!("{}", e)
        }
    }

    getdata(port_name, baud_rate, path);
}
