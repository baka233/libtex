use std::collections::{HashMap, BTreeSet, VecDeque, BTreeMap};
use std::io::{BufRead};
use crate::token::Token;
use log::{info, warn, debug};
use dot;
use std::borrow::Cow;

/// Grammer struct have all grammers store in it
pub struct Grammer {
    grammers : BTreeMap<Token, Vec<Vec<Token>>>,
    pattern : BTreeMap<Token, Vec<(Token, Vec<Token>)>>,
    // use BTreeSet store the ans
    first : HashMap<Token, BTreeSet<Token>>,
    follow : HashMap <Token, BTreeSet<Token>>, 
    start_token : Token,
    terminal_set : BTreeSet<Token>,
    nonterminal_set : BTreeSet<Token>,
    ll_map : BTreeMap<Token, BTreeMap<Token, Option<(Token, Vec<Token>)>>>
}

impl Grammer {
    /// read each grammer by line
    pub fn new<R : BufRead>(start_token : Token, reader : &mut R) -> Self {
        let mut buf = String::new();
        let mut grammers = BTreeMap::new();
        let mut pattern = BTreeMap::new();
        let mut terminal_set = BTreeSet::new();
        let mut nonterminal_set = BTreeSet::new();
        while reader.read_line(&mut buf).unwrap() != 0 {
            let data : Vec<_> = buf.split("->").map(str::trim).collect();
            debug!("{:?}", data);
            if data.len() != 2 {
                panic!("Wrong grammer!");
            }
            let token = Token::parse_token(data.get(0).unwrap()).next().unwrap();
            nonterminal_set.insert(token.clone());
            let token_vec : Vec<_> = Token::parse_token(data.get(1).expect("the grammer didn't exist!")).collect();
            // insert into patterns
            for each in &token_vec {
                if each.is_non_terminal() {
                    nonterminal_set.insert(each.clone());
                    if pattern.get(each).is_none() {
                        pattern.insert(each.clone(), Vec::new());
                    }
                    pattern.get_mut(each).unwrap().push((token.clone(), token_vec.clone()));
                } else if each.is_terminal() {
                    terminal_set.insert(each.clone());
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
        let follow = Self::follow(&grammers, &pattern, &first, start_token.clone());
        let ll_map = Self::ll(&grammers, &first, &follow, &terminal_set, &nonterminal_set); 

        
        Grammer {
            grammers,
            pattern,
            first,
            follow,
            start_token,
            terminal_set,
            nonterminal_set, 
            ll_map,
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
                            .push_back(
                                grammer
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
        first    : &HashMap<Token, BTreeSet<Token>>,
        start    : Token
    ) -> HashMap<Token, BTreeSet<Token>> {
        let mut ans = HashMap::new();    
        let mut unmaped : HashMap<Token, VecDeque<(Token, VecDeque<Token>)>> = HashMap::new();

        // prepare ans and unmaped data
        for item in grammers.keys() {
            ans.insert(item.clone(), BTreeSet::new());
            if *item == start {
                ans.get_mut(item).unwrap().insert(Token::Epsilon);
            }
            // use vecdeque for pop_front and push_front
            unmaped.insert(item.clone(), VecDeque::new());
        }

        for (item_key, item_value) in pattern {
            for (token, grammer) in item_value {
                for i in (0..grammer.len()).rev() {
                    if *grammer.get(i).unwrap() == *item_key {
                        // if grammer is last element
                        if i == grammer.len() - 1 {
                            unmaped.get_mut(item_key).unwrap().push_back((token.clone(), VecDeque::new()));
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

            debug!("------------new round-------------");
            for item_key in pattern.keys() {
                if let Some((token, mut top)) = unmaped.get_mut(item_key).unwrap().pop_front() {
                    if top.len() == 0 {
                        let ans_vec  : Vec<_> = ans.get(&token).unwrap().iter().cloned().collect();
                        let unmaped_vec : Vec<_> = unmaped.get(&token).unwrap().iter().cloned().collect();
                        for each in ans_vec {
                            ans.get_mut(&item_key).unwrap().insert(each);
                        }
                        for each in unmaped_vec {
                            unmaped.get_mut(&item_key).unwrap().push_back(each);
                        }
                        continue;
                    }
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
                            let ans_vec  : Vec<_> = ans.get(&token).unwrap().iter().cloned().collect();
                            let unmaped_vec : Vec<_> = unmaped.get(&token).unwrap().iter().cloned().collect();
                            for each in ans_vec {
                                ans.get_mut(&item_key).unwrap().insert(each);
                            }
                            for each in unmaped_vec {
                                unmaped.get_mut(&item_key).unwrap().push_back(each);
                            }
                        }
                    }

                }

                debug!("item key {:?} ans is {:?}", item_key, ans.get(&item_key).unwrap());
                debug!("item key {:?} unmaped is {:?}", item_key, unmaped.get(&item_key).unwrap());

            }
        }

        ans
    }

    pub fn ll(
        grammers        : &BTreeMap<Token, Vec<Vec<Token>>>,
        first           : &HashMap<Token, BTreeSet<Token>>,
        follow          : &HashMap <Token, BTreeSet<Token>>, 
        terminal_set    : &BTreeSet<Token>,
        nonterminal_set : &BTreeSet<Token>,
    ) -> BTreeMap<Token, BTreeMap<Token, Option<(Token, Vec<Token>)>>>  {
        let mut ans = BTreeMap::new();
        for item_key in nonterminal_set {
            let mut tmp = BTreeMap::new();
            for terminal in terminal_set {
                tmp.insert(terminal.clone(), None);
            }
            tmp.insert(Token::Epsilon, None);
            ans.insert(item_key.clone(), tmp);
            debug!("ans {:?} is {:?}", item_key, ans.get(item_key));
        }
        for (item_key, item_value) in grammers {
            for grammer in item_value {
                let token = grammer.get(0).unwrap().clone();
                if token == Token::Epsilon {
                    // if grammer is epsilon, use follow
                    for each in follow.get(item_key).unwrap() {
                        *ans.get_mut(item_key) 
                            .unwrap()
                            .get_mut(each)
                            .unwrap() = Some((item_key.clone(), grammer.clone()));
                    }
                } else if token.is_terminal() {
                    *ans.get_mut(item_key)
                        .unwrap()
                        .get_mut(&token)
                        .unwrap() = Some((item_key.clone(), grammer.clone()));
                } else if token.is_non_terminal() {
                    for each in first.get(item_key).unwrap() {
                        *ans.get_mut(item_key)
                            .unwrap()
                            .get_mut(each) 
                            .unwrap() = Some((item_key.clone(), grammer.clone()));
                    }
                }
            }
        }
        ans
    }

    pub fn draw(&self, s : &str) -> GrammerTree {
        let mut stack = Vec::new(); 
        let mut token_vec : VecDeque<_> = Token::parse_token(s)
            .collect();
        token_vec.push_back(Token::Epsilon);

        // push the end to the Stack
        stack.push(Token::Epsilon);
        stack.push(self.start_token.clone());
        let mut node = GrammerNode::new(self.start_token.clone());
        tree_scanner(&mut node, &mut stack, &mut token_vec, &self.ll_map);
        GrammerTree {
            tree : Some(node)
        }
    }

    pub fn analysis(&self, s : &str) {
        let mut stack = Vec::new(); 
        let mut token_vec : VecDeque<_> = Token::parse_token(s)
            .collect();
        token_vec.push_back(Token::Epsilon);

        // push the end to the Stack
        stack.push(Token::Epsilon);
        stack.push(self.start_token.clone());
        
        while !stack.is_empty() {
            print!("{:<20}", print_table(stack.iter()));
            let top = stack.pop().expect("unreachable");
           
            print!("{:<20}", print_table(token_vec.iter()));
            let top_token = token_vec.get(0).unwrap();
            if top == Token::Epsilon {
                if top == *top_token {
                    println!("Success");
                    break;
                } else {
                    println!("Error A");
                }
            }
            if top.is_terminal() {
                if *top_token == top {
                    let tmp = token_vec.pop_front();
                    println!("{}匹配", tmp.unwrap());
                } else {
                    println!("Error C");
                    break;
                }
            } else if top.is_non_terminal() {
                if let Some((_, data)) = self.ll_map.get(&top).unwrap().get(&top_token).unwrap() {
                    for item in data.iter().rev() {
                        if *item == Token::Epsilon {
                            break;
                        }  
                        stack.push(item.clone());
                    }
                    print!("{:<20}", print_table(data.iter()));
                    println!("");
                } else {
                    println!("Error D");
                    break;
                }
            }
        }
    }
}

fn tree_scanner(root : &mut GrammerNode, stack : &mut Vec<Token>, token_vec : &mut VecDeque<Token>, ll_map : &BTreeMap<Token, BTreeMap<Token, Option<(Token, Vec<Token>)>>>) {
    if stack.is_empty() {
        return;
    }
    let top = stack.pop().expect("unreachable");
    let top_token = token_vec.get(0).unwrap();
    println!("top is {}", top);
    if top == Token::Epsilon {
        if top == *top_token {
            println!("Success");
            return;
        } else {
            println!("Error A");
            return;
        }
    }
    if top.is_terminal() {
        if *top_token == top {
            let tmp = token_vec.pop_front().unwrap();
            println!("{}匹配", tmp);
            return;
        } else {
            println!("Error C");
            return;
        }
    } else if top.is_non_terminal() {
        if let Some((_, data)) = ll_map.get(&top).unwrap().get(&top_token).unwrap() {
            for item in data.iter().rev() {
                if *item == Token::Epsilon {
                    return;
                }  
                stack.push(item.clone());
            }
            for item in data.iter() {
                let mut node = GrammerNode::new(item.clone());
                tree_scanner(&mut node, stack, token_vec, ll_map);
                root.child.push(node);
            }
            print!("{:<20}", print_table(data.iter()));
            println!("");
        } else {
            println!("Error D");
            return;
        }
    }
}

#[derive(Debug)]
pub struct GrammerTree {
    tree : Option<GrammerNode>,
}

impl GrammerTree {
    pub fn new(token : Token) -> Self {
        GrammerTree {
            tree : Some(GrammerNode::new(token))
        }
    }

}

type Nd = (Token, usize);
type Ed = ((Token, usize), (Token, usize));


impl<'a> dot::GraphWalk<'a, Nd, Ed> for GrammerTree {
    fn nodes(&self) -> dot::Nodes<'a, Nd> {
        let root = self.tree.as_ref().unwrap();
        let mut queue = VecDeque::new() ;
        let mut num = 0;
        let mut nodes = Vec::new();
        queue.push_back((root, num));
        nodes.push((root.token.clone(), num));
        while !queue.is_empty() {
            let (top, _) = queue.pop_front().unwrap();
            top.child.iter().for_each(|item| {
                num += 1;
                queue.push_back((item, num)) ;
                nodes.push((item.token.clone(), num));
            });
        }
        Cow::Owned(nodes)
    } 

    fn edges(&'a self) -> dot::Edges<'a, Ed> {
        let root = self.tree.as_ref().unwrap();
        let mut queue = VecDeque::new() ;
        let mut num = 0;
        let mut edges = Vec::new();
        queue.push_back((root, num));
        while !queue.is_empty() {
            let (top, tmp_num) = queue.pop_front().unwrap();
            top.child.iter().for_each(|item| {
                num += 1;
                edges.push(((top.token.clone(), tmp_num), (item.token.clone(), num)));
                queue.push_back((item, num));
            });
        }
        Cow::Owned(edges)
    }

    fn source(&self, e : &Ed) -> Nd { e.0.clone() }
    fn target(&self, e : &Ed) -> Nd { e.1.clone() }
}

impl<'a> dot::Labeller<'a, Nd, Ed> for GrammerTree {
    fn graph_id(&'a self) -> dot::Id<'a> {
        dot::Id::new("Grammer").unwrap()
    }

    fn node_id(&'a self, n : &Nd) -> dot::Id<'a> {
        dot::Id::new(format!("node{}", n.1)).expect("format error")
    }

    fn node_label(&self, nd : &Nd) -> dot::LabelText {
        dot::LabelText::LabelStr(Cow::Owned(nd.0.to_string()))
    }

//    fn node_shape(&'a self, n : &Nd) -> Option<dot::LabelText<'a>> {
//        if self.nodes.get(n).unwrap().nodetype == DFANodeType::NonTerminal {
//            Some(dot::LabelText::LabelStr(Cow::Owned("circle".to_string())))
//        } else {
//            Some(dot::LabelText::LabelStr(Cow::Owned("box".to_string())))
//        }
//    }

//    fn edge_label(&self, ed : &Ed) -> dot::LabelText {
//        let s = match ed.2 {
//            Token::Epsilon => "Epsilon".to_string(),
//            Token::Character(ch) => ch.to_string(),
//            _ => unreachable!(),
//        };
//        dot::LabelText::LabelStr(Cow::Owned(format!("{}", s)))
//    }
}



#[derive(Debug)]
pub struct GrammerNode {
    token : Token,
    child : Vec<GrammerNode> 
}

impl GrammerNode {
    pub fn new(token : Token) -> Self {
        GrammerNode {
            token,
            child : Vec::new()
        }
    }
}

#[test]
fn test_first_1() {
    //simple_logger::init().unwrap();
    use std::io::{BufReader};
    use std::fs::File;
    let grammers = Grammer::new(Token::NonTerminal("S".to_string()), &mut BufReader::new(&mut File::open("test.in").unwrap()));
    for (key, value) in &grammers.first {
        println!("FIRST({:?}) = {:?}", key, value);
    }
    for (key, value) in &grammers.follow {
        println!("FOLLOW({:?}) = {:?}", key, value);
    }

}

#[test]
fn test_first_2() {
    simple_logger::init().unwrap();
    use std::io::{BufReader};
    use std::fs::File;
    let grammers = Grammer::new(Token::NonTerminal("E".to_string()), &mut BufReader::new(&mut File::open("test2.in").unwrap()));
    for (key, value) in &grammers.first {
        println!("FIRST({:?}) = {:?}", key, value);
    }
    for (key, value) in &grammers.follow {
        println!("FOLLOW({:?}) = {:?}", key, value);
    }
    print!("{:<20}", "");
    for item in &grammers.terminal_set {
        print!("{:<20}", format!("{}", item));
    }
    println!("");
    for (key, value) in &grammers.ll_map {
        print!("{:<20}", format!("{}", key));
        for item in &grammers.terminal_set {
            if let Some((item_key, item_value)) = value.get(item).unwrap() {
                print!("{:<20}", print_table(item_value.iter()));
            } else {
                print!("{:<20}", "Error");
            }
        }
        if let Some((item_key, item_value)) = value.get(&Token::Epsilon).unwrap() {
            print!("{:<20}", print_table(item_value.iter()));
        } else {
            print!("{:<20}", "Error");
        }

        println!("");
    }
    println!("{:-<120}", "-");
    grammers.analysis("i+i*i");
}


#[test]
fn test_first_3() {
    simple_logger::init().unwrap();
    use std::io::{BufReader};
    use std::fs::File;
    let grammers = Grammer::new(Token::NonTerminal("E".to_string()), &mut BufReader::new(&mut File::open("test3.in").unwrap()));
    for (key, value) in &grammers.first {
        println!("FIRST({:?}) = {:?}", key, value);
    }
    for (key, value) in &grammers.follow {
        println!("FOLLOW({:?}) = {:?}", key, value);
    }
    print!("{:<20}", "");
    for item in &grammers.terminal_set {
        print!("{:<20}", format!("{}", item));
    }
    println!("");
    for (key, value) in &grammers.ll_map {
        print!("{:<20}", format!("{}", key));
        for item in &grammers.terminal_set {
            if let Some((item_key, item_value)) = value.get(item).unwrap() {
                print!("{:<20}", print_table(item_value.iter()));
            } else {
                print!("{:<20}", "Error");
            }
        }
        if let Some((item_key, item_value)) = value.get(&Token::Epsilon).unwrap() {
            print!("{:<20}", print_table(item_value.iter()));
        } else {
            print!("{:<20}", "Error");
        }

        println!("");
    }
    println!("{:-<120}", "-");
    grammers.analysis("(i*i)+i");
    let grammer_draw = grammers.draw("(i*i)+i");
    let mut output = File::create("example.dot").unwrap();
    dot::render(&grammer_draw, &mut output).unwrap();

}


fn print_table<'a, I>(iter : I) -> String
where 
    I : Iterator<Item = &'a Token>,
{
    let mut string = String::new();
    for each in iter {
        string = format!("{}{}", string, each);
    }
    string
}


fn map_string(s : &str) -> String {
    match s {
        "+" => "add".to_owned(),
        "-" => "minus".to_owned(),
        "*" => "mul".to_owned(),
        "/" => "div".to_owned(),
        "(" => "l_bracets".to_owned(),
        ")" => "r_bracets".to_owned(),
        _ => s.to_string(),
    }
}
