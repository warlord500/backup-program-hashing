extern crate data_encoding;
extern crate ring;

use data_encoding::HEXUPPER;
use ring::digest::{Context, Digest, SHA256};
use std::fs::File;
use std::io::{BufReader, Read,Result};
use std::path::Path;
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
/* At this point I have not decided how i am going to deal with the hashes.
	my first thought is to put them all in a big hashset to deal with
	the hashset contains a tuple of (hash,file_name) 
	which would work but i wondering how big this is gonna get because i need this to be faster than
	rsync by a lot. 
	if it was faster, and dealt with organization better than i would be far less infuriated by it. 
	every system, i have seen just deals with the timing and scheduling stuff. 
	which would be nice to work with too. but that is beyond the scope of this project.
	
  
*/
fn generate_cache(the_path: &Path,  cache : &mut HashMap<String,String>) -> Result<()> { 
	for entry in std::fs::read_dir(the_path)? {
			let entry = entry?;
			let path = entry.path();
			if filter_path(&path) {
				if path.is_file() {
					 let before = Instant::now();
					 
					 let digest = sha256_digest(BufReader::new(File::open(&path)?))?;
					 
					 let duration  = Instant::now().duration_since(before).as_millis();
					 let file_name = path.file_stem().unwrap().to_string_lossy();
					 
					 println!("processing: {} in {}",file_name, duration);
					 
					cache.insert(HEXUPPER.encode(digest.as_ref()), file_name.into());
					 
				} else  if path.is_dir() {
					generate_cache(path.as_path(),cache)?
				} else {
					//do something when we have a shortcut
				}
			}
			
		}
		Ok(())
}
fn proccess_one_way(src_to_move : HashSet<String>,
					cache_set   :  HashMap<String,String>,
					dest_cache_set   :  HashMap<String,String>) -> Result<()>

fn main() -> Result<()> {
	let src_folder = "E:\\programming\\test\\incremental_backup_test_data\\src";
	let dest_folder = "E:\\programming\\test\\incremental_backup_test_data\\dest";
	
	let mut cache = HashMap::<String,String>::new();
	generate_cache(Path::new(src_folder),&mut cache)?;
	
	let mut dest_cache = HashMap::<String,String>::new();
	generate_cache(Path::new(dest_folder), &mut dest_cache)?;
	
	
	// if this program is set to one way move
	// dest_to_move is ignored!!
	let (src_to_move,dest_to_move)  = {
		//get all the hashes
		let cache_set = cache.keys().collect::<HashSet<_>>();
		let dest_cache_set = dest_cache.keys().collect::<HashSet<_>>();
		
		//difference of hashes
		let srcs_to_move = cache_set.difference(&dest_cache_set).cloned().collect::<HashSet<_>>();
		let dest_to_move = dest_cache_set.difference(&cache_set).cloned().collect::<HashSet<_>>();
		(srcs_to_move,dest_to_move)
	};
	
	/*proccessing hash list */
	
	for item in src_to_move {
		let src = cache.get(item).unwrap();
		let dest = dest_cache.get(item).unwrap();
		fs::copy(src,dest)?;
	}
	
	Ok(())
}