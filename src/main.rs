extern crate data_encoding;
extern crate ring;
extern crate lapp;

use data_encoding::HEXUPPER;
use ring::digest::{Context, Digest, SHA256};
use std::fs::File;
use std::io::{BufReader, Read,Result};
use std::path::{Path,PathBuf};
use std::time::Instant;
use std::collections::{HashMap,HashSet};
use std::fs;


fn sha256_digest<R: Read>(mut reader: R) -> Result<Digest> {
    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 1024];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }

    Ok(context.finish())
}

fn filter_path(the_path : &Path) -> bool {
	//check if the file is actually hidden or contains . or _ before 
	// the name!
	let filename = the_path.file_stem().unwrap().to_string_lossy();
	let hidden = filename.starts_with(|x|  match x {
		'.' | '_' => true,
		_ => false,
	});
	// you want to filter hidden items
	!hidden 
}
/* use mutable reference so that dont generate mutiple hashmaps just a single one
	when this code is ran recursively.
*/ 
fn generate_cache(the_path       : &Path,  
				  cache          : &mut HashMap<String,String>,
			      shouldRecurse  : bool) -> Result<()> {
 	
	for entry in std::fs::read_dir(the_path)? {
			let entry = entry?;
			let path = entry.path();
			if filter_path(&path) {
				if path.is_file() {
					 let before = Instant::now();
					 
					 let digest = sha256_digest(BufReader::new(File::open(&path)?))?;
					 
					 let duration  = Instant::now().duration_since(before).as_millis();
					 let file_name = path.file_name().unwrap().to_string_lossy().into();
					 
					 println!("processing: {} in {}",file_name, duration);
					 
					cache.insert(HEXUPPER.encode(digest.as_ref()), file_name);
					 
				} else  if path.is_dir() && shouldRecurse {
					generate_cache(path.as_path(),cache,shouldRecurse)?
				} else {
					//do something when we have a shortcut
				}
			}
			
		}
		Ok(())
}
fn proccess_one_way(src_to_move : HashSet<&String>,
					cache       : &HashMap<String,String>, 
					dest_cache  : &HashMap<String,String>, 
					src_loc     : &mut PathBuf,
					dest_loc    : &mut PathBuf) {
	/*proccessing hash list */
	
	for item in src_to_move {
		let org_name = cache.get(&*item).unwrap();
		src_loc.push(org_name);
		dest_loc.push(org_name);
		match fs::copy(&src_loc,&dest_loc) {
			Err(_e) => {
				// write!(stderr, e.message()) ugh 
			},
			Ok(_) => {
				src_loc.pop();
				dest_loc.pop();
			}
		}
	}
}

fn main() -> Result<()> {
	let args = lapp::parse_args("
	A quick backup program that is faster than rsync for a large number of files.
	-r, --disable_recursion 
	-w, --one_way 
	<src> (path) the source directory important in one_way mode.
	<dest> (path) the destination
	");
	
	let mut  src_folder = args.get_path_result("src").expect("source argument to be vaild");
	let mut  dest_folder = args.get_path_result("dest").expect("destination argument to be valid");
	let should_recurse = !args.get_bool("disable_recursion"); //normally run recursively
	let one_way = args.get_bool("one_way");
	
	let mut cache = HashMap::<String,String>::new();
	generate_cache(&*src_folder,&mut cache,should_recurse)?;
	let cache = cache;
	
	let mut dest_cache = HashMap::<String,String>::new();
	generate_cache(&*dest_folder, &mut dest_cache,should_recurse)?;
	let dest_cache = dest_cache;
	
	// if this program is set to one way move;
	// dest_to_move is ignored!!
	let (src_to_move,_dest_to_move)  = {
		//get all the hashes
		let cache_set = cache.keys().collect::<HashSet<_>>();
		let dest_cache_set = dest_cache.keys().collect::<HashSet<_>>();
		
		//difference of hashes
		let srcs_to_move = cache_set.difference(&dest_cache_set).cloned().collect::<HashSet<_>>();
		let dest_to_move = dest_cache_set.difference(&cache_set).cloned().collect::<HashSet<_>>();
		(srcs_to_move,dest_to_move)
	};
	
	/*proccessing hash list */
	print!("{:?}", src_to_move);
	proccess_one_way(src_to_move,&cache, &dest_cache,&mut src_folder,&mut dest_folder);
	Ok(())
}