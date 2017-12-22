// < begin copyright > 
// Copyright Ryan Marcus 2017
// 
// This file is part of basicaf.
// 
// basicaf is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
// 
// basicaf is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
// 
// You should have received a copy of the GNU General Public License
// along with basicaf.  If not, see <http://www.gnu.org/licenses/>.
// 
// < end copyright > 
use std::collections::{HashMap, HashSet};
use ir::blockgen::{Block};




pub fn find_dominated_nodes(stmts: &[Block], idx: usize)
                            -> HashSet<usize>
{
    let mut unreachable = HashSet::new();
    // do a DFS from the root block without block idx
    // any node we don't reach is dominated by block idx

    for i in 0..stmts.len() {
        unreachable.insert(i);
    }

    if idx == 0 {
        // root dominates all
        return unreachable;
    }
    
    let mut stack = Vec::new();
    stack.push(0);

    while !stack.is_empty() {
        let v = stack.pop().unwrap();
        if unreachable.contains(&v) && v != idx {
            // v has not yet been discovered
            unreachable.remove(&v);
            for blk_idx in stmts[v as usize].out_blocks.iter() {
                stack.push(*blk_idx);
            }
        }
    }

    return unreachable;
}

pub fn build_dominated_sets(stmts: &[Block])
                            -> HashMap<usize, HashSet<usize>>
{
    let mut to_r = HashMap::new();

    for i in 0..stmts.len() {
        to_r.insert(i, find_dominated_nodes(stmts, i));
    }
    
    return to_r;
}

pub fn get_reverse_postorder(stmts: &[Block])
                             -> HashMap<usize, usize>
{
    let mut to_r = HashMap::new();

    let mut stack = Vec::new();
    let mut unvisited = HashSet::new();

    for i in 0..stmts.len() {
        unvisited.insert(i);
    }
    
    stack.push(0);
    while !stack.is_empty() {
        let v = stack.pop().unwrap();
        if unvisited.contains(&v) {
            // v has not yet been discovered
            unvisited.remove(&v);
            let curr_len = to_r.len();
            to_r.insert(v, curr_len);
            for blk_idx in stmts[v as usize].out_blocks.iter() {
                stack.push(*blk_idx);
            }
        }
    }

    return to_r;

}

pub fn get_back_edges(stmts: &[Block],
                      rpo: &HashMap<usize, usize>,
                      dom: &HashMap<usize, HashSet<usize>>)
                      -> HashSet<(usize, usize)>
{
    let mut to_r = HashSet::new();
    
    for (src, b) in stmts.iter().enumerate() {
        for dst in b.out_blocks.iter() {
            let src_rpo = rpo.get(&src).unwrap();
            let dst_rpo = rpo.get(dst).unwrap();
            if src_rpo >= dst_rpo {
                // this is a retreating edge.
                // it is also a back edge if dest dominates
                // idx.
                let dest_dominates = dom.get(dst).unwrap();
                if dest_dominates.contains(&src) {
                    // this is a back edge.
                    to_r.insert((src, *dst));
                }
                
            }
        }
    }

    return to_r;
}


enum DFSColor {
    White, Gray, Black
}

fn ensure_reducable(stmts: &[Block],
                    back_edges: &HashSet<(usize, usize)>,
                    colors: &mut HashMap<usize, DFSColor>,
                    idx: usize) {

    // check to see if the graph is cyclic, without the
    // back edges.

    colors.insert(idx, DFSColor::Gray);
    for child in stmts[idx].out_blocks.iter() {
        let tup = (idx, *child);
        if back_edges.contains(&tup) {
            continue;
        }
        
        match *colors.get(child).unwrap() {
            DFSColor::Black => { },

            DFSColor::White => {
                ensure_reducable(stmts, back_edges,
                                 colors, *child);
            },

            DFSColor::Gray => {
                panic!("non-reducible flow -- check lines \
                        {} -> {}",
                       get_last_cmd!(stmts[idx]).ln,
                       get_last_cmd!(stmts[*child]).ln);
            }
        };
    }

    colors.insert(idx, DFSColor::Black);
}


fn reachable(stmts: &[Block],
             known_loop_nodes: &HashSet<usize>,
             dest: usize,
             src: usize,
             without_passing: usize) -> bool {
    
    // do a DFS to see if we can reach endpoint without
    // going through the header.
    let mut visited = HashSet::new();
    let mut stack = Vec::new();
    stack.push(src);
    while !stack.is_empty() {
        let v = stack.pop().unwrap();
        visited.insert(v);
        if known_loop_nodes.contains(&v) {
            // we reached a node that can reach the endpoint
            // so we can too.
            return true;
        }

        if v == dest {
            return true;
        }

        for child in stmts[v].out_blocks.iter() {
            if visited.contains(child) {
                // we've already seen it
                continue;
            }

            if *child == without_passing {
                // can't go this way...
                continue ;
            }

            stack.push(*child);
                
        }
    }

    return false;
}
             

fn get_nodes_for_back_edge(stmts: &[Block],
                           edge: &(usize, usize))
                           -> HashSet<usize>
{
    let mut loop_nodes = HashSet::new();
    let (endpoint, header) = *edge;
    loop_nodes.insert(endpoint);
    loop_nodes.insert(header);

    for i in 0..stmts.len() {
        if loop_nodes.contains(&i) {
            continue; // already part of the loop!
        }

        if reachable(stmts, &loop_nodes,
                     endpoint, i, header) {
            loop_nodes.insert(i);
        } else {
            // check to see if we can reach a return wthout
            // going through the header. if we can, then
            // that return statement ends this loop
            // (as well as any others it is in)
            
            
        }
      
    }
    
    return loop_nodes;
}

fn collect_loop_exits(stmts: &[Block],
                      loop_nodes: &HashSet<usize>)
                      -> Vec<usize>
{
    let mut to_r = Vec::new();

    for n in loop_nodes.iter() {
        for out_node in stmts[*n].out_blocks.iter() {

            if !loop_nodes.contains(out_node) &&
                !to_r.contains(out_node)
            {
                to_r.push(*out_node);
            }
        }
    }

    return to_r;
}
                         


pub fn eliminate_gotos(stmts: &mut Vec<Block>) {
    let dominated = build_dominated_sets(stmts);
    let rpo = get_reverse_postorder(stmts);
    let back_edges = get_back_edges(stmts, &rpo, &dominated);

    let mut colors = HashMap::new();
    for i in 0..stmts.len() {
        colors.insert(i, DFSColor::White);
    }
    ensure_reducable(stmts, &back_edges,
                     &mut colors, 0);


    for ed in back_edges.iter() {
        let loop_nodes = get_nodes_for_back_edge(stmts, ed);
        let exit_nodes = collect_loop_exits(stmts, &loop_nodes);
        let (_end, header) = *ed;

        let mut loop_block = Block::new_loop(exit_nodes, loop_nodes);
        let lp_idx = stmts.len();
        loop_block.out_blocks.push(header);

        let inblocks= stmts[header].in_blocks.clone();
        for incoming in inblocks {
            
            let idx = stmts[incoming].out_blocks.iter()
                .position(|&r| r == header)
                .expect("Incoming and outgoing edges not set correctly!");


            stmts[incoming].out_blocks.remove(idx);
            stmts[incoming].out_blocks.push(lp_idx);

        }


        stmts[header].in_blocks.clear();
        stmts[header].in_blocks.push(lp_idx);
        stmts.push(loop_block);
    }


    
}
