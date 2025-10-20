use crate::topo::layout::VisualGraph;

// Name tree refers the nested hierarchy of subgraphs and when ordering nodes with functions that has word tree
// this tree is taken into accoutn for detaills see the paper by sander

fn update_col_map(
    vg: &VisualGraph,
    col_map: &mut Vec<usize>,
    layer_idx: usize,
) {
    for (j, n) in vg.dag.ranks()[layer_idx].iter().enumerate() {
        col_map[n.get_index()] = j;
    }
}

pub(crate) fn do_it_inner(vg: &mut VisualGraph, col_map: &mut Vec<usize>) {
    // print all nodes with their names
    let layers = vg.dag.num_levels();
    vg.dag.subgraph_order_by_p_layerwise(0);
    update_col_map(vg, col_map, 0);
    // update col_map using last row
    for i in 1..layers {
        vg.dag.node_order_wp_layerwise(i, col_map);
        vg.dag.subgraph_order_by_p_layerwise(i);
        update_col_map(vg, col_map, i);
    }
    let lambda_order = vg.dag.subgraph_order_by_p();

    // clear each layer and refill it according to lambda order
    for layer_idx in 0..layers {
        vg.dag.ranks_mut()[layer_idx].clear();
    }
    for n in lambda_order {
        let lvl = vg.dag.level(n);
        vg.dag.ranks_mut()[lvl].push(n);
    }
    vg.dag.subgraph_order_by_p_layerwise(layers - 1);
    update_col_map(vg, col_map, layers - 1);
    for i in (0..layers - 1).rev() {
        vg.dag.node_order_ws_layerwise(i, col_map);
        vg.dag.subgraph_order_by_p_layerwise(i);
        update_col_map(vg, col_map, i);
    }

    let lambda_order = vg.dag.subgraph_order_by_p();

    // clear each layer and refill it according to lambda order
    for layer_idx in 0..layers {
        vg.dag.ranks_mut()[layer_idx].clear();
    }
    for n in lambda_order {
        let lvl = vg.dag.level(n);
        vg.dag.ranks_mut()[lvl].push(n);
    }
}

pub fn prepare_borders(vg: &mut VisualGraph) {
    let subgraph_levels = vg.dag.get_subgraph_levels();
    vg.dag.place_horizontal_borders(&subgraph_levels);
    vg.dag.place_vertical_borders();
    // insert Placeholder nodes for borders
    for s in 0..vg.dag.num_subgraphs() {
        let (lvl_start, lvl_end) = subgraph_levels[s];
        vg.add_vertical_border(lvl_start, lvl_end);
    }
}

const SANDERS_ITERATIONS: usize = 4;

#[cfg_attr(not(feature = "log"), allow(unused_assignments, unused_variables))]
pub(crate) fn do_it(vg: &mut VisualGraph) {
    let mut column_map = vec![0; vg.dag.len()];
    for r in vg.dag.ranks().iter() {
        for (j, n) in r.iter().enumerate() {
            column_map[n.get_index()] = j;
        }
    }
    for _ in 0..SANDERS_ITERATIONS {
        do_it_inner(vg, &mut column_map);
    }
    prepare_borders(vg);
}
