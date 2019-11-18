use std::collections::{HashMap, BTreeSet, VecDeque, BTreeMap};
use std::io::{BufRead};
use crate::token::Token;
use log::{info, warn, debug};

/// Grammer struct have all grammers store in it
pub struct Grammer {
    grammers : BTreeMap<Token, Vec<Vec<Token>>>,
    pattern : BTreeMap<Token, Vec<(Token, Vec<Token>)>>,
    // use BTreeSet store the ans
    first : HashMap<Token, BTreeSet<Token>>,
    follow : HashMap <Token, BTreeSet<Token>>, 
}

impl Grammer {
    /// read each grammer by line
    pub fn new<R : BufRead>(reader : &mut R) -> Self {
        let mut buf = String::new();
        let mut grammers = BTreeMap::new();
        let mut pattern = BTreeMap::new();
        while reader.read_line(&mut buf).unwrap() != 0 {
            let data : Vec<_> = buf.split("->").map(str::trim).collect();
            debug!("{:?}", data);
            if data.len() != 2 {
                panic!("Wrong grammer!");
            }
            let token = Token::parse_token(data.get(0).unwrap()).next().unwrap();
            let token_vec : Vec<_> = Token::parse_token(data.get(1).expect("the grammer didn't exist!")).collect();
            // insert into patterns
            for each in &token_vec {
                if each.is_non_terminal() {
                    if pattern.get(each).is_none() {
                        pattern.insert(each.clone(), Vec::new());
                    }
                    pattern.get_mut(each).unwrap().push((token.clone(), token_vec.clone()));
                }
            }
            if grammers.get(&token).is_none() {
                grammers.insert(token.clone(), Vec::new());
            }
            grammers.get_mut(&token).unwrap().push(token_vec);
            buf.clear();
        }

        // add empty pattern
        for key in grammers.keys() {
            if pattern.get(key).is_none() {
                pattern.insert(key.clone(), Vec::new());
            }
        }

        for item in &grammers {
            debug!("{:?}", item);
        }

        let first = Self::first(&grammers);
        let follow = Self::follow(&grammers, &pattern, &first);

        
        Grammer {
            grammers,
            pattern,
            first,
            follow,
        }
    }
}

impl Grammer {
    /// get first set of current Grammer
    fn first(grammers : &BTreeMap<Token, Vec<Vec<Token>>>) -> HashMap<Token, BTreeSet<Token>> {
        let mut ans : HashMap<Token, BTreeSet<Token>> = HashMap::new();
        let mut unmaped : HashMap<Token, VecDeque<VecDeque<Token>>> = HashMap::new();

        for key in grammers.keys() {
            ans.insert(key.clone(), BTreeSet::new()); 
            unmaped.insert(key.clone(), VecDeque::new());
        }
            

        // init ans and unmaped
        for (item_key, item_value) in grammers {
            for grammer in item_value {
                let token = grammer.get(0).unwrap();
                match token {
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
            for item_key in grammers.keys() {
                debug!("-----------{:?}----------", item_key);
                if let Some(mut top) = unmaped.get_mut(&item_key).unwrap().pop_front() {
                    flag = false;
                    debug!("{:?} top is {:?}", item_key, top);
                    if *top.get(0).expect("this grammer should have some data, but didn't") == *item_key {
                        continue;
                    }
                    
                    // if only if the top has at least one element
                    let first_token = top.get(0).unwrap();

                    let mut epsilon_flag = false;
                    let ans_vec : Vec<_> = ans.get(&first_token)
                        .unwrap()
                        .iter()
                        .cloned()
                        .collect();
                    for each_token in ans_vec {
                        if each_token == Token::Epsilon {
                            epsilon_flag = true; 
                            continue;
                        }
                        ans.get_mut(&item_key).unwrap().insert(each_token);
                    }

                    
                    let unmaped_vec : Vec<_> = unmaped.get(&first_token)
                        .unwrap()
                        .iter()
                        .cloned()
                        .collect();
                    for each_token in unmaped_vec {
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


    /// Get follow set from current grammers
    pub fn follow(
        grammers : &BTreeMap<Token, Vec<Vec<Token>>>,
        pattern  : &BTreeMap<Token, Vec<(Token, Vec<Token>)>>,
        first    : &HashMap<Token, BTreeSet<Token>>
    ) -> HashMap<Token, BTreeSet<Token>> {
        let mut ans = HashMap::new();    
        let mut unmaped : HashMap<Token, VecDeque<(Token, VecDeque<Token>)>> = HashMap::new();

        // prepare ans and unmaped data
        for item in grammers.keys() {
            ans.insert(item.clone(), BTreeSet::new());
            // use vecdeque for pop_front and push_front
            unmaped.insert(item.clone(), VecDeque::new());
        }

        for (item_key, item_value) in pattern {
            for (token, grammer) in item_value {
                for i in (0..grammer.len()).rev() {
                    if *grammer.get(i).unwrap() == *item_key {
                        // if grammer is last element
                        if i == grammer.len() - 1 {
                            ans.get_mut(item_key).unwrap().insert(Token::Epsilon);
                        } else if grammer.get(i+1).unwrap().is_terminal() {
                            ans.get_mut(item_key).unwrap().insert(grammer[i+1].clone());
                        } else {
                            let tmp = (token.clone(), grammer[i+1..].iter().cloned().collect());
                            unmaped.get_mut(item_key).unwrap().push_back(tmp);
                        }
                    }
                }
            }
            debug!("token {:?} pattern is {:?}", item_key, item_value);
            debug!("token {:?} is {:?}", item_key, ans.get(item_key));
            debug!("token {:?} unmaped is {:?}", item_key, unmaped.get(item_key));

        }

        let mut flag = false;
        loop {
            if flag {
               break; 
            }
            flag = true;

            for item_key in pattern.keys() {
                if let Some((token, mut top)) = unmaped.get_mut(item_key).unwrap().pop_front() {
                    let first_token = top.pop_front().unwrap();
                    if !first_token.is_non_terminal() {
                        panic!(format!("unmaped token must be NonTerminal, but its {:?}", token.clone()));
                    }
                    if first_token == *item_key {
                        continue;
                    }
                    flag = false;
                    
                    let mut has_epsilon = false;
                    for each_item in first.get(&first_token).unwrap() {
                        if each_item.is_epsilon()  {
                            has_epsilon = true;
                            continue;
                        }
                        ans.get_mut(item_key).unwrap().insert(each_item.clone());
                    }

                    if has_epsilon == true {
                        top.pop_front();        
                        if let Some(head_token) = top.get(0) {
                            if head_token.is_terminal() {
                                ans.get_mut(item_key).unwrap().insert(head_token.clone());  
                            } else {
                                unmaped.get_mut(item_key).unwrap().push_front((token.clone(), top.clone()));
                            }
                        } else {
                            ans.get_mut(item_key).unwrap().insert(Token::Epsilon);
                        }
                    }

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
    for (key, value) in &grammers.first {
        println!("FIRST({:?}) = {:?}", key, value);
    }
    for (key, value) in &grammers.follow {
        println!("FOLLOW({:?}) = {:?}", key, value);
    }

}
