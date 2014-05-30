
use std::io::{File, BufferedReader, IoResult };
use std::string::String;

enum TokenReaderState {
	TopLevel,
	Comment,
	Identifier,
	Preprocessor,
	String,
	SpecialChar
}

//let keywords = ["class", "void", "int", "struct", "public:", "protected:", "private:"];

fn isSpecialChar( c : char ) -> bool {
	c == ';' || c == ',' || c == '{' || c == '}' || c == '(' || c == ')' || c == '<' || c == '>' || c == '&' || c == '*'
}

fn isNewLine(c : char) -> bool {
	c == '\n' || c == '\r'
}

fn isWhiteSpace( c : char ) -> bool {
	c == ' ' || c =='\t' || isNewLine(c)
}


struct TokenReader {
	reader : BufferedReader<File>,
	buffer : String,
	state  : TokenReaderState,
	pending : Vec<char>,
}

impl TokenReader {

	fn new(file: File ) -> TokenReader {
		TokenReader { 
			reader : BufferedReader::new(file),
			buffer : String::new(),
			state : TopLevel,
			pending : Vec::new()
		}
	}
	
	fn setState( &mut self, state : TokenReaderState) {
		self.buffer.truncate(0);

		self.state = state;
	}

	fn nextChar( &mut self ) -> IoResult<char> {
		match self.pending.pop() {
			Some(c) => Ok(c),
			None	=> self.reader.read_char()
		}
	}

	fn discardNext( &mut self ) {
		self.nextChar();
	}

	fn peekChar(&mut self) -> IoResult<char> {
		match self.nextChar() {
			Ok(c) => {
				self.putBack(c);
				Ok(c)
			},
			Err(c) => Err(c)
		}
	}

	fn putBack( &mut self, c : char ) {
		self.pending.push(c)
	}

	fn emit( &mut self ) -> Option<String> {
		let token = Some( self.buffer.to_string() );
		self.buffer.truncate(0);
		token
	}

	fn append(&mut self, c : char ) {
		self.buffer.push_char(c)
	}

	fn appendAndEmit(&mut self, c : char ) -> Option<String> {
		self.append(c);
		self.emit()
	}
}

impl Iterator<String> for TokenReader {

	fn next(&mut self) -> Option<String> {
		self.setState( TopLevel );

		loop {
			match self.state {
				TopLevel => {
					match self.peekChar() {
						Ok(c) if isWhiteSpace(c) 	=> self.discardNext(),
						Ok(c) if isSpecialChar(c)	=> self.setState( SpecialChar ),
						Ok(c) if c == '#'			=> self.setState( Preprocessor ),
						Ok(c) if c == '"'			=> self.setState( String ),
						Ok(c) if c == '/'			=> self.setState( Comment ),
						Ok(_)						=> self.setState( Identifier ),
						Err(_) 						=> return None
					}
				},
				Preprocessor => {
					match self.nextChar() {
						Ok(c) if isWhiteSpace(c)	=> return self.emit(),
						Ok(c) 					=> self.append(c),
						Err(_)	=> return None
					}	
				},

				String => {
					match self.nextChar() {
						Ok(c) => {
							self.append(c);

							if (c == '"' && self.buffer.len() > 1) || isNewLine(c) {
								return self.emit()
							}
						},
						Err(_) => return None
					}
				},

				Comment => {
					match self.nextChar() {
						Ok(c) if isNewLine(c) 	=> self.setState(TopLevel),
						Ok(_)					=> (),
						Err(_)					=> return None
					}
				},

				Identifier => {
					match self.nextChar() {
						Ok(c) if isWhiteSpace(c) || isSpecialChar(c) => {
							self.putBack(c);
							return self.emit();
						},
						Ok(c)					=> self.append(c),
						Err(_)					=> return None
					}
				},

				SpecialChar => {
					let c = self.nextChar().unwrap();
					return self.appendAndEmit( c );
				}
			}			
		}
	}
}

fn parsePragma(tokens : & Vec<String>, mut idx: uint) -> uint {
	idx + 1
}

fn parseInclude(tokens : & Vec<String>, mut idx: uint) -> uint {
	println!("Included: {}", tokens.get(idx));
	idx + 1
}

fn parseDefine(tokens : & Vec<String>, mut idx: uint) -> uint {
	fail!("define needs line endings?")
}

fn parseInheritedClasses( tokens : & Vec<String>, mut idx: uint) -> uint {
	fail!("TODO");
}

fn parseClass( tokens : & Vec<String>, mut idx: uint ) -> uint {

	let name = tokens.get(idx).as_slice();
	let divider = tokens.get(idx+1).as_slice();

	idx += 2;

	if divider == ";" {
		println!("Declared class {}", name);
	}
	else if divider == "{" || divider == ":" {
		println!("Defined class: {}", name);

		if divider == ":" {
			idx = parseInheritedClasses(tokens, idx);
		}

		idx = parseScope(tokens, idx) + 1; // skip ;
	}

	idx
}

fn optional(  attribute : & str, tokens : & Vec<String>, idx: uint ) -> uint {
	if tokens.get(idx).as_slice() == attribute {
		idx + 1
	}
	else { 
		idx
	}
}

fn parseType( tokens : & Vec<String>, mut idx: uint) -> uint {

	idx = optional( "const", tokens, idx );

	let typename = tokens.get(idx).as_slice();
	idx += 1;

	let ptr = tokens.get(idx).as_slice();
	if ptr == "*" || ptr == "&" {
		println!("Referenced {}", typename );
		idx += 1
	}
	else {
		println!("Used {}", typename );
	}

	let delim = tokens.get(idx).as_slice();

	if delim == "," {
		idx = parseType(tokens,idx+1);
	}
	else if delim == "<" {
		idx = parseType(tokens,idx+1) + 1;
	}

	idx
}

// fn rest( tokens : & Vec<String>, mut idx: uint ) -> uint {
// 	while idx < tokens.len() {
// 		println!("{}", tokens.get(idx));
// 		idx += 1;
// 	}
// 	idx
// }

fn parseTypedef( tokens : & Vec<String>, mut idx: uint ) -> uint {

	idx = parseType(tokens, idx);

	println!("Defined type {}", tokens.get(idx));

	idx + 2
}

fn parseIdentifier( tokens : & Vec<String>, mut idx: uint ) -> uint {
	
	idx = optional("static", tokens, idx);

	parseType(tokens, idx) + 1
}

fn parseFunctionSignature( tokens : & Vec<String>, mut idx: uint ) -> uint {

	let name = tokens.get(idx-2);
	loop {
		let delim = tokens.get(idx).as_slice();

		if delim == ")" {
			idx += 1;
			break;
		}
		else if delim == "," {
			idx += 1;
		}

		idx = parseIdentifier(tokens, idx);
	}

	idx = optional("const", tokens, idx);
	idx = optional("override", tokens, idx);

	let delim = tokens.get(idx).as_slice();
	idx += 1;
	if delim == ";" {
		println!("Declared function {}", name);
	}
	else {
		fail!("Definition of functions not supported");
	}

	idx
}


fn parseMember( tokens : & Vec<String>, mut idx: uint ) -> uint {

	idx = parseIdentifier(tokens, idx) - 1;

	let name = tokens.get(idx);
	idx += 1;

	let delim = tokens.get(idx).as_slice();
	idx += 1;

	if delim == "(" {
		idx = parseFunctionSignature(tokens, idx);
	}
	else {
		println!("Declared member {}", name);
	}
	idx
}

fn parseScope( tokens : & Vec<String>, mut idx: uint ) -> uint {

	while idx < tokens.len() {

		let keyword = tokens.get(idx).as_slice();
		idx += 1;

		if keyword == "}" {
			break;
		}
		if keyword == "#pragma" {
			idx = parsePragma(tokens, idx);
		}
		else if keyword == "#include" {
			idx = parseInclude(tokens, idx);
		}
		else if keyword == "#define" {
			idx = parseDefine(tokens, idx);
		}
		else if keyword == "class" || keyword == "struct" {
			idx = parseClass(tokens, idx);
		}
		else if keyword == "typedef" {
			idx = parseTypedef(tokens, idx);
		}
		else if keyword == "public:" || keyword == "protected:" || keyword == "private:" {
			//nothing
		}
		else { //dunno!
			idx = parseMember(tokens, idx);
		}
	}
	return idx;
}

fn parseFile(path: &Path) {

	let file = File::open(path).unwrap();
	let mut tokenizer = TokenReader::new(file);
	let mut tokens: Vec<String> = Vec::new();

	for token in tokenizer {
		tokens.push(token);
	}

	println!("Parsing {}", path.filename_display() );

	parseScope( &tokens, 0 );

}


fn main() {
	let path = Path::new("/Users/tommaso/DEV/Minecraftpe/handheld/src/locale");

	for dir in std::io::fs::walk_dir(&path).unwrap() {
		match dir.extension_str() {
			Some(ext) if ext == "h" => parseFile(&dir),
			_ => ()
		}
	}
}