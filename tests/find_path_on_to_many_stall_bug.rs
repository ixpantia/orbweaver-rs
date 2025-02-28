use ntest_timeout::timeout;
use orbweaver::prelude::*;

const FIND_PATH_GRAPH_WITH_BUG: &str = "./assets/find_path_on_to_many_error.txt";

fn get_graph() -> DirectedGraph {
    use std::io::BufRead;
    let mut builder = DirectedGraphBuilder::new();
    std::io::BufReader::new(
        std::fs::File::open(FIND_PATH_GRAPH_WITH_BUG)
            .expect("Unable to read find_path_on_to_many_error.txt"),
    )
    .lines()
    .map_while(Result::ok)
    .for_each(|l| {
        if let Some((parent, child)) = l.split_once('\t') {
            builder.add_edge(parent, child);
            builder.add_edge(child, parent);
        }
    });

    builder.build_directed()
}

#[test]
#[timeout(5000)] // Test fails if it takes more than 5000ms
fn find_path_on_to_many_does_not_stall() {
    let dg = get_graph();

    let result = dg.find_path_one_to_many(
        "17443",
        [
            "7146", "10265", "11376", "20580", "18238", "3869", "10472", "20291", "10799", "19928",
            "19703", "13753", "19505", "3795", "17367", "4641", "18943", "12925", "17420", "8238",
            "8829", "15860", "5465", "277", "10602", "11688", "15716", "3046", "16173", "13793",
            "19694", "6087", "17854", "20669", "4184", "20639", "14252", "17661", "15257", "9418",
            "2435", "4486", "2127", "20931", "14587", "12439", "11112", "12438", "10294", "17787",
            "1150", "6845", "18976", "2448", "3745", "19051", "813", "6919", "20454", "5198",
            "8126", "20784", "3285", "17843", "3747", "10767", "6098", "8047", "13030", "11667",
            "15769", "10344", "2781", "16010", "10366", "9307", "17810", "18259", "18102", "15003",
            "10423", "3203", "16050", "11330", "2096", "16268", "16239", "1512", "15394", "13383",
            "10111", "20131", "9638", "273", "17931", "7399", "14932", "5609", "6428", "17959",
        ],
    );

    assert!(result.is_ok());
}
