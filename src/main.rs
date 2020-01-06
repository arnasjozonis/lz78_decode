use std::fs::{File};
use std::io::{BufReader,BufWriter};
use bitbit::{BitWriter,BitReader};
use bitbit::reader::MSB;
use std::collections::HashMap;
use std::iter::Iterator;
use std::io::prelude::*;
use std::env;
use std::time::{SystemTime};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Please provide filename argument!");
        return;        
    }
    let filename = &args[1];
    let file = File::open(filename.to_string()).unwrap();
    let buff_reader = BufReader::new(file);
    let mut br: BitReader<_,MSB> = BitReader::new(buff_reader);

    let newfile = filename.trim_right_matches(".lz");
    let w = File::create(newfile).unwrap();
    let mut buf_writer = BufWriter::new(w);

    //read if there is dict size limit
    let dict_len_limit = br.read_byte().unwrap();
    let dict_len_limit = 2usize.pow(dict_len_limit as u32);
    println!("Dict len limit: {}", dict_len_limit);

    let mut dictionary = Tree::new();
    let first_byte = br.read_byte().unwrap();
    buf_writer.write(&[first_byte]).unwrap();
    dictionary.add_node(first_byte, 0);
    let mut counter = dictionary.nodes.len();

    loop {
        let max_idx = counter - 1;
        let dict_idx_bits_length = log2(max_idx) as usize;
        let parent_node = match br.read_bits(dict_idx_bits_length) {
            Ok(bits) => bits as usize,
            _ => break
        };
        let entry_char = match br.read_byte() {
            Ok(byte) => byte,
            _ => break
        };
        let mut chars_buffer = vec![entry_char];
        let mut chars = dictionary.get_chars_from_parents(parent_node);
        chars_buffer.append(&mut chars);
        chars_buffer.reverse();
        buf_writer.write(&chars_buffer[..]).unwrap();
        dictionary.add_node(entry_char, parent_node);
        counter += 1;

        if dict_len_limit != 1 && dictionary.nodes.len() == (dict_len_limit + 1) {
            dictionary = Tree::new();
            let first_byte =  match br.read_byte() {
                Ok(byte) => byte,
                _ => break
            };
            buf_writer.write(&[first_byte]).unwrap();
            dictionary.add_node(first_byte, 0);
            counter = dictionary.nodes.len();
        }
    }

    buf_writer.flush().unwrap();
}

#[derive(Debug, Clone)]
struct Node {
    parent_node: usize,
    children: HashMap<u8, usize>,
    index: usize,
    value: u8
}

fn log2(number: usize) -> u32 {
    64 - number.leading_zeros()
}

struct Tree {
    nodes: Vec<Node>
}

impl Tree {
    pub fn new() -> Tree {
        let mut nodes: Vec<Node> = Vec::new();
        let root_node = Node {
            value: 0,
            index: 0,
            parent_node: 0,
            children: HashMap::new()
        };
        nodes.push(root_node);
        Tree {
            nodes
        }
    }

    pub fn add_node(&mut self, value: u8, parent_node: usize) -> usize {
        let index = self.nodes.len();
        self.nodes.push(Node {
            value,
            index,
            parent_node,
            children: HashMap::new()
        });
        if let Some(pn) = self.nodes.get_mut(parent_node) {
            match (*pn).children.get(&value) {
                None => {(*pn).children.insert(value, index);},
                _ => {}
            }
        }
        index
    }

    pub fn get_chars_from_parents(&self, parent_node: usize) -> Vec<u8> {
        let mut result = Vec::new();
        let mut current_parent = match self.nodes.get(parent_node) {
            Some(parent) => parent,
            _ => {
                println!("fail with pn: {}", parent_node); 
                self.nodes.get(0).unwrap()
            }
        };

        loop {
            if current_parent.index == 0 {
                break;
            }
            result.push(current_parent.value);
            current_parent = self.nodes.get(current_parent.parent_node).unwrap();
        }

        result
    }
}
