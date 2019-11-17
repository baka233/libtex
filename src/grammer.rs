use std::collections::{HashMap, BTreeSet, VecDeque, BTreeMap};
use std::io::{BufRead};
use crate::token::Token;
use log::{info, warn, debug};

/// Grammer struct have all grammers store in it
pub struct Grammer {
    grammers : BTreeMap<Token, Vec<Vec<Token>>>,
}

impl Grammer {
    /// read each grammer by line
    pub fn new<R : BufRead>(reader : &mut R) -> Self {
        let mut buf = String::new();
        let mut grammers = BTreeMap::new();
        while reader.read_line(&mut buf).unwrap() != 0 {
            let data : Vec<_> = buf.split("->").map(str::trim).collect();
            debug!("{:?}", data);
            if data.len() != 2 {
                panic!("Wrong grammer!");
            }
            let token = Token::parse_token(data.get(0).unwrap()).next().unwrap();
            let token_vec : Vec<_> = Token::parse_token(data.get(1).expect("the grammer didn't exist!")).collect();
            if grammers.get(&token).is_none() {
                grammers.insert(token.clone(), Vec::new());
            }
            grammers.get_mut(&token).unwrap().push(token_vec);
            buf.clear();
        }

        for item in &grammers {
            debug!("{:?}", item);
        }
        
        Grammer {
            grammers
        }
    }
}

impl Grammer {
    /// get first set of current Grammer
    pub fn first(&self) -> HashMap<Token, BTreeSet<Token>> {
        let mut ans = HashMap::new();
        let mut unmaped : HashMap<Token, VecDeque<VecDeque<Token>>> = HashMap::new();

        for key in self.grammers.keys() {
            ans.insert(key.clone(), BTreeSet::new()); 
            unmaped.insert(key.clone(), VecDeque::new());
        }
            

        // init ans and unmaped
        for (item_key, item_value) in &self.grammers {
            for grammer in item_value {
                let token = grammer.get(0).unwrap();
                let _ = match token {
                    Token::Terminal(_) | Token::Epsilon => {
                        ans.get_mut(item_key).unwrap().insert(token.clone());
                    },
                    Token::NonTerminal(_) => {
                        unmaped.get_mut(item_key)
                            .unwrap()
                            .push_back(grammer
                                .iter()
                                .cloned()
                                .collect()
                            );
                    },
                };
            }
            debug!("ans of {:?} is {:?}", item_key,  ans.get(item_key));
            debug!("unmaped of {:?} is {:?}", item_key,  unmaped.get(item_key));
        }

        let mut flag = false;
        loop {
            if flag == true {
                break;
            }
            // set flag to true, if there are any change to the grammers, we will set it to false,
            // continue the loop
            flag = true;
            for item_key in self.grammers.keys() {
                debug!("-----------{:?}----------", item_key);
                if let Some(mut top) = unmaped.get_mut(&item_key).unwrap().pop_front() {
                    flag = false;
                    debug!("{:?} top is {:?}", item_key, top);
                    if *top.get(0).expect("this grammer should have some data, but didn't") == *item_key {
                        continue;
                    }
                    
                    // if only if the top has at least one element
                    let first_token = top.get(0).unwrap();

                    let vec : Vec<_> = ans.get_mut(&first_token).unwrap().iter().cloned().collect();
                    let mut epsilon_flag = false;
                    vec.iter()
                        .for_each(|each_token| {
                            if *each_token == Token::Epsilon {
                                epsilon_flag = true; 
                                return;
                            }
                            ans.get_mut(&item_key).unwrap().insert(each_token.clone());
                        });

                    let unmaped_vec : Vec<_> = unmaped.get_mut(&first_token).unwrap().iter().cloned().collect();
                    for each_token in unmaped_vec.iter() {
                        let mut tmp = top.clone();
                        tmp.pop_front().unwrap();
                        tmp.push_front(each_token.get(0).unwrap().clone());
                        unmaped.get_mut(&item_key)
                            .unwrap()
                            .push_back(tmp);
                    }
                    
                    if epsilon_flag == true {
                        top.pop_front(); 
                        if top.len() != 0 {
                            unmaped.get_mut(&item_key).unwrap().push_front(top);
                        } else {
                            ans.get_mut(&item_key).unwrap().insert(Token::Epsilon);
                        }
                    }
                    info!("after ans is {:?}", ans.get(item_key));
                    info!("after unmaped is {:?}", unmaped.get(item_key));
                }
            }
        }
        ans
    }
}

#[test]
fn test_first() {
    simple_logger::init().unwrap();
    use std::io::{BufReader};
    use std::fs::File;
    let grammers = Grammer::new(&mut BufReader::new(&mut File::open("test.in").unwrap()));
    for (key, value) in &grammers.first() {
        println!("FIRST({:?}) = {:?}", key, value);
    }
}
