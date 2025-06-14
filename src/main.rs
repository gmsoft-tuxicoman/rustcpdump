extern crate getopts;
use getopts::Options;
use pcap::{Capture,Linktype};
use std::env;

pub mod proto;

use crate::proto::ProtoParser;
use crate::proto::ProtoStackEntry;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}


fn main() {


    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optopt("r", "read", "input PCAP file", "NAME");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]){
        Ok(m) => { m }
        Err(f) => { panic!("{}", f.to_string()) }
    };


    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }


    let filename = match matches.opt_str("r") {
        None => { panic!("No filename provided") }
        Some(f) => { f }
    };

    let mut cap = Capture::from_file(filename).unwrap();


    let datalink = cap.get_datalink();
    println!("Capture datalink : {:?}", datalink);

    while let Ok(packet) = cap.next_packet() {

        process_packet(packet.data, datalink);
    }

}


fn process_packet<'a>(data: &'a[u8], lt: Linktype) {
    
    assert_eq!(lt, Linktype(1));

    let mut t = proto::ProtoNumberType::Pcap;
    let mut n = lt.0 as u32;
    let mut data = data;

    let mut stack = Vec::new();
    loop {
        let p_res = proto::get_next_proto(t, n, data);
        let mut p = match p_res {
            Ok(p) => p,
            _ => break,
        };
        let res =  p.process();
        match res {
            Ok(proto_slice) => {
                t = proto_slice.number_type;
                n = proto_slice.number;
                data = &data[proto_slice.start .. proto_slice.end];
                stack.push(ProtoStackEntry{parser: p, parse_result: true});
            },
            Err(()) => {
                stack.push(ProtoStackEntry{parser: p, parse_result: true});
                break
            }
        }
        
    }
    let mut prev_layer : Option<Box<dyn ProtoParser>> = None;
    for p in stack {
        match prev_layer {
            None => p.parser.print(None),
            Some(l) => p.parser.print(Some(&l))
        }
        prev_layer = Some(p.parser);
    }
    println!()
}
