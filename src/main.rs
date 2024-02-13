use std::{net::{TcpListener, TcpStream, IpAddr}, io::{Write, Read}, str::FromStr, fmt::Debug};
use termion::screen::IntoAlternateScreen;
use local_ip_address::local_ip;
fn main() {

	//get option client or server
    let mut input_line = String::new();
	let stdin = std::io::stdin();
	let mut stdout = std::io::stdout().into_alternate_screen().unwrap();
	print!("Server or Client[s/C]: ");
	stdout.flush().unwrap();
	stdin.read_line(&mut input_line).unwrap();
	input_line = input_line.to_ascii_lowercase().replace("\n", "");
	let is_client= input_line!="s".to_string();
	input_line.clear();

	//bind/connect
	let (ip,port,client_addr);
	let mut tcp_stream;
	if !is_client {
		ip = local_ip().unwrap();
		let socket = TcpListener::bind((ip,0)).unwrap();
        println!("serving on {:?}",socket.local_addr().unwrap());
        print!("awaiting connection... ");
        stdout.flush().unwrap();
		(tcp_stream, client_addr) = socket.accept().expect("Nope");
		println!("Connection from {}",client_addr);
	} else {
		ip = get_ip();		
        port=get_port();
		tcp_stream = TcpStream::connect((ip,port)).unwrap();
	}
	
    //server picks pick random first player
	let mut plr:u8;
	if !is_client {
		plr = if rand::random() {1} else {2};//player 1's turn(50/50 bc bool is only true or false)
		tcp_stream.write(&[plr]).unwrap();
		println!("{} is first!", if plr==1 {"Server"} else {"Client"});
	} else {
		let mut buf: [u8;1] = [0];
		tcp_stream.read(&mut buf).unwrap();
		plr = buf[0];
	}
	let plr_num = if is_client {2} else {1};


	//actually run the game now
	let mut board: [[u8;3];3];
    let mut x:u8;//the winning player or 0
	let mut full:bool;
	let mut rematch:bool=true;
	while rematch {
        //reset eatch match
        x=0;
        full=false;
        board=[[b'.';3];3];
		while x==0&&!full  {
            println!("{}{}'s turn",termion::clear::All, if plr==1 {'x'} else {'o'});
			//do plr turn
			if plr==plr_num {
				println!("Yo it's my turn");
                print_board(&board);
				let mut pos_enum: Pos = input("Which square[tl/t/tr/l/c/r/bl/b/br]: ");
				let mut pos:u8 = pos_enum as u8;
				let mut y:usize = (pos / 3).into();
				let mut x:usize = (pos % 3).into();
				while board[y][x]==b'x'||board[y][x]==b'o' {
					pos_enum = input("Try again[tl/t/tr/l/c/r/bl/b/br]: ");
					pos = pos_enum as u8;
					y = (pos / 3).into();
					x = (pos % 3).into();
				}
				board[y][x]=if is_client {b'o'} else {b'x'};
				println!("x:{} y:{}",x,y);
				send_board(&mut tcp_stream, &board);

			} else {
                print_board(&board);
				println!("Getting board/awaiting move");
				get_board(&mut tcp_stream, &mut board);
			}
            //next player's turn
			plr = if plr==1 {2} else {1};
			x=check_win(&board);
			full=check_full(&board);
		}//while currently playing
        
        //show winning move
        print!("{}",termion::clear::All);
        println!("plr {} is the winner!", char::from(x));
        print_board(&board);

        //should we have a rematch?
		let rematch_enum : YN= input("Rematch[y/n]:");
		rematch=match rematch_enum {
			YN::Y => true,
			YN::N => false
		};
        let mut buf:[u8;1]=[0];
        //can't both 
        //if is_client {
            tcp_stream.write(&[rematch as u8]).unwrap();
            tcp_stream.read(&mut buf).unwrap();
        //} else {
        //    tcp_stream.read(&mut buf).unwrap();
        //    tcp_stream.write(&[rematch as u8]).unwrap();
        //}
        rematch=rematch&&buf[0]!=0;
	} //while rematch
	tcp_stream.shutdown(std::net::Shutdown::Both).unwrap();
	println!("Hello, world!");
}

#[test]
fn test_basic_check_win() {
	assert_eq!(check_win(&[[b'x',b'x',b'x'],[0,0,0],[0,0,0]]),b'x');
	assert_eq!(check_win(&[[0,0,0],[b'x',b'x',b'x'],[0,0,0]]),b'x');
	assert_eq!(check_win(&[[0,0,0],[0,0,0],[b'x',b'x',b'x']]),b'x');
	assert_eq!(check_win(&[[0,0,b'x'],[0,0,b'x'],[0,0,b'x']]),b'x');
	//the empty squares can be anything except b'x' or b'o'
    assert_eq!(check_win(&[[b'o',b'.',b'x'],[b'x',b'o',b'.'],[b'.',b'.',b'o']]),b'o');

}
#[test]
fn test_empty() {
	assert_eq!(check_win(&[[0,0,0],[0,0,0],[0,0,0]]),0);
	assert_eq!(check_win(&[[b'.';3];3]),0);
}

#[test]
fn test_multiple_on_baord() {
	assert_eq!(check_win(&[[b'x',b'o',b'o'],[0,0,b'o'],[0,0,b'o']]),b'o');
}
#[test]
fn test_check_full() {
	assert_eq!(check_full(&[[b'x',b'x',b'o'];3]),true);
	assert_eq!(check_full(&[[b'x',b'o',b'.'];3]),false);
	assert_eq!(check_full(&[[b'.';3];3]),false);
}

#[derive(Debug)]
enum YN {
	Y,
	N,
}
impl FromStr for YN {
	type Err = String;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"y" => Ok(YN::Y),
			"n" => Ok(YN::N),
			_ => Err("Yes or No[y/n]: {}".to_string().replace("{}", s)),
		}
	}
}
#[derive(Debug)]
enum Pos {
	TL=0,
	T=1,
	TR=2,
	L=3,
	C=4,
	R=5,
	BL=6,
	B=7,
	BR=8
}
impl FromStr for Pos {
	type Err = String;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"tl" => Ok(Pos::TL),
			"t"  => Ok(Pos::T ),
			"tr" => Ok(Pos::TR),
			"l"  => Ok(Pos::L ),
			"c"  => Ok(Pos::C ),
			"r"  => Ok(Pos::R ),
			"bl" => Ok(Pos::BL),
			"b"  => Ok(Pos::B ),
			"br" => Ok(Pos::BR),
			_ => Err("Pick a valid spot. you picked: {}".to_string().replace("{}", s)),
		}
	}
}

fn print_board(board: &[[u8;3];3]) {
	print!("-----\n ");
	for y in 0..3 {
		print!("{}",String::from_utf8_lossy(&board[y]));
		print!("\n ");
	}
	println!("\x08-----\n\n");
}
fn check_full(board: &[[u8;3];3]) -> bool {
	for y in 0..3 {
		for x in 0..3 {
			if board[y][x]!=b'x'&&board[y][x]!=b'o' {//if any empty spaces
				return false;//then exit
			}
		}
	}
	return true;//found no empty spaces
}

//returns b'x', b'o' if win or else 0
fn check_win(board: &[[u8;3];3]) -> u8 {
    for p in [b'x',b'o'] {
        //horizontal wins
        for y in board {
            if  y[0]==p&& 
                y[1]==p&& 
                y[2]==p
            {return p;}
        }
        //veritical
        for x in 0..3 {
            if  board[0][x]==p &&
                board[1][x]==p &&
                board[2][x]==p 
            {return p;}
        }
        //diagonal
        if  board[0][0]==p &&
            board[1][1]==p &&
            board[2][2]==p
        {return p;}
        if  board[0][2]==p &&
            board[1][1]==p &&
            board[2][0]==p
        {return p;}
    }
    return 0;
}

fn input<T: FromStr + std::fmt::Debug>(mesg:&str) -> T where <T as FromStr>::Err: core::fmt::Debug {
	let mut input_line = String::new();
	let stdin = std::io::stdin();
	let mut stdout = std::io::stdout();
	print!("{}",mesg);
	stdout.flush().unwrap();
	stdin.read_line(&mut input_line).expect("failed to readline");
	let mut line_result:Result<T, _> = input_line.trim().parse();
	while !line_result.is_ok() {
		input_line.clear();
		println!("Invalid!");
		println!("{:?}",line_result);
		print!("{}",mesg);
		stdout.flush().unwrap();
		stdin.read_line(&mut input_line).expect("failed to readline");
		line_result = input_line.trim().parse();
	}
	return line_result.expect("XP");
}
fn get_port() -> u16 {
	let mut input_line = String::new();
	let stdin = std::io::stdin();
	let mut stdout = std::io::stdout();
	print!("port: ");
	stdout.flush().unwrap();
	stdin.read_line(&mut input_line).expect("failed to readline");
	let mut line_result:Result<u16, _> = input_line.trim().parse();
	while !line_result.is_ok() {
		input_line.clear();
		println!("Invalid!");
		println!("{:?}",line_result);
		print!("port: ");
		stdout.flush().unwrap();
		stdin.read_line(&mut input_line).expect("failed to readline");
		line_result = input_line.trim().parse();
	}
	return line_result.expect("XP");
}
fn get_ip() -> IpAddr {
	let mut input_line = String::new();
	let stdin = std::io::stdin();
	let mut stdout = std::io::stdout();
	print!("Server IP: ");
	stdout.flush().unwrap();
	stdin.read_line(&mut input_line).expect("failed to readline");
	let mut line_result:Result<IpAddr, _> = input_line.trim().parse();
	while !line_result.is_ok() {
		input_line.clear();
		println!("Invalid!");
		println!("{:?}",line_result);
		print!("Server IP: ");
		stdout.flush().unwrap();
		stdin.read_line(&mut input_line).expect("failed to readline");
		line_result = input_line.trim().parse();
	}
	return line_result.expect("XP");
}
fn send_board(s:&mut TcpStream,board: &[[u8;3];3]) {
	let mut tmp: [u8;9]=[0u8;9];
	for y in 0..3 {
		for x in 0..3 {
			tmp[y*3+x]=board[y][x];
		}
	}
	s.write(&tmp).expect("couldn't write");
}
fn get_board(s:&mut TcpStream,board: &mut [[u8;3];3]) {
	let mut tmp:[u8;10]=[
		board[0][0],board[0][1],board[0][2],
		board[1][0],board[1][1],board[1][2],
		board[2][0],board[2][1],board[2][2],b'\n'
		];
	s.read(&mut tmp).unwrap();
	println!("[debug] get_board:{:?}",tmp);
	for y in 0..3 {
		for x in 0..3 {
			board[y][x]=tmp[y*3+x];
		}
	}
}
/*
[
	[ x x x ],
	[x x x ],
	[ x x x ],
]
*/
