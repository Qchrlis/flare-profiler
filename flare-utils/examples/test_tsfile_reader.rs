
extern crate rand;
extern crate chrono;

use flare_utils::ValueType;
use flare_utils::timeseries::*;
use chrono::Local;
use rand::Rng;

fn main() {
    let mut tsfile = TimeSeriesFileReader::new("tsfile-test1").unwrap();
    let info = tsfile.get_header_info();
    println!("tsfile header: {:?}", info);

    let start_time = info.begin_time;
    let unit_time = 100 as i64;

    for i in 0..100 {
        test_get_range_value(&mut tsfile, start_time, unit_time);
    }
}

fn test_get_range_value(tsfile: &TimeSeriesFileReader, start_time: i64, unit_time: i64) {
    let info = tsfile.get_header_info();
    let mut rng = rand::thread_rng();
    let start = rng.gen_range(0, info.amount/2) as i64;
    let end = rng.gen_range(info.amount/2, info.amount) as i64;
    let ratio = 2i32.pow(rng.gen_range(0, 5));

    let t1 = Local::now().timestamp_millis();
    let tsresult = tsfile.get_range_value(start_time + unit_time * start, start_time + unit_time * end, ratio * unit_time as i32);
    let t2 = Local::now().timestamp_millis();
    println!("result: begin_time: {}, end_time:{}, unit_time: {}, steps: {}, cost: {}ms",
             tsresult.begin_time, tsresult.end_time, tsresult.unit_time, tsresult.steps, (t2 - t1));
}