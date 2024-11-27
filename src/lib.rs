use regex::Regex;


mod tests {
	use std::{collections::HashMap, iter::Map, ops::Range, vec};

use regex::RegexBuilder;

use super::*;
	#[test]
    fn regex_test() {

		let re = Regex::new(r":([^\/]*):").unwrap();
		let hay = ("/transaction/:transaction_id:/:transaction_item_id:/*").to_string();
		let mut route_regex:String = hay.clone().replace("/", "\\/");
		let mut ranges:Vec<Range<usize>> = vec![];
		let mut path_parameter_map: HashMap<&String, String> = HashMap::new();
		println!("{}",route_regex);
		let captured_params = re.captures_iter(&route_regex.as_str()).
			filter_map( 
				|fm| {
					
					fm.get(1).map( 
						|m| {
							println!("{:#?}", m);
							ranges.push(m.range());
							m.as_str().to_string()
						}
					)
				}
			).collect::<Vec<String>>();
		
		ranges.iter().rev().for_each(|r: &Range<usize>| {
			route_regex.replace_range(r.start-1..r.end+1, "([^\\/]*)");
		});
		
		if route_regex.ends_with("\\/*"){

			let len = route_regex.len();
			route_regex.replace_range(len-1..len, "(.+)");
		}
		let created_regex = RegexBuilder::new(route_regex.as_str()).build().unwrap();
		//let mut path_parameters_map = HashMap::new();

		
		let captured_strings  = created_regex.captures_iter("/transaction/sample_transaction_id/sample_transaction_item_id/sample_extension/extension_extra").into_iter().map( |capture_entry|{ 	
			capture_entry.iter().map(| capture_group: Option<regex::Match<'_>>|{
				capture_group.unwrap().as_str()
			}).collect::<Vec<&str>>()
		}).collect::<Vec<_>>().get(0).unwrap().to_owned();

		
		
		//test regex hhere
		captured_params.iter().enumerate().for_each(|(index, key)|{
			path_parameter_map.insert(key, captured_strings[index+1].to_owned());
		});
		println!("{:#?}", path_parameter_map);
		
    }
}