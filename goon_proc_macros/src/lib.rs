
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, AttributeArgs, Block, Expr, Field, Ident, Item, ItemFn, Lit, LitStr, NestedMeta, Token, Type};
#[proc_macro]
pub fn goon_update(input: TokenStream) -> TokenStream {
    let two_args = parse_macro_input!(input as TwoArgs);
    let name = two_args.ident;
    let data = two_args.ident2;
    quote!{
        goon_node_update(&#name,#data);
    }.into()
}
#[derive(Clone)]
struct TwoArgs{
    ident: Ident,
    t1: Token![,],
    ident2: Ident,
}
impl syn::parse::Parse for TwoArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            ident: input.parse()?,
            t1: input.parse()?,
            ident2: input.parse()?,
        })
    }
}

#[proc_macro]
pub fn global(input: TokenStream) -> TokenStream {
    let old_ident = parse_macro_input!(input as Ident);
    let ident = Ident::new(&format!("{}",&old_ident.to_string().to_lowercase()), String::new().span());

    quote!{
        let #ident = #old_ident.clone();
    }.into()
}
#[proc_macro]
pub fn declare_global(input: TokenStream) -> TokenStream {
    let vars = parse_macro_input!(input with syn::punctuated::Punctuated<TypedMacroInput, syn::Token![;]>::parse_terminated);
    let (expanded, expanded_2): (Vec<_>,Vec<_>) = vars
        .iter()
        .map(|var_name_lit| {
            let var_name = var_name_lit.ident.clone();
            let ident = Ident::new(&format!("f_{}",&var_name), String::new().span());
            let u_ident = Ident::new(&format!("fu_{}",&var_name), String::new().span());
            let var_name_s = var_name_lit.ident.to_string();
            let ident_type = var_name_lit.ident_type.clone();
            let val = var_name_lit.val.clone();
            let expanded = quote! {
                fn #ident() {
                    if let Ok(mut node) = NODE.clone().lock() {
                        node.add(&#var_name_s.to_string());
                    };
                }
                #ident();
            };
            let expanded_2 = quote! {
                fn #u_ident(data: &[u8]) {
                    if let Ok(mut var) = #var_name.clone().lock() {
                        if let Ok(s) = String::from_utf8(data.to_vec()) {
                            if let Ok(val) = serde_json::from_str(&s) {
                                *var = val;
                            }
                        }
                    };
                }
                lazy_static::lazy_static!{
                    static ref #var_name: Global<#ident_type> = Global::new(#val);
                }
            };
            
            (expanded,expanded_2)
        })
        .unzip();
    let (expanded_3, expanded_4): (Vec<_>,Vec<_>) = vars
                                 .clone()
        .iter()
        .map(|var_name_lit| {
            let var_name = var_name_lit.ident.clone();
            let ident = Ident::new(&format!("f_{}",&var_name), String::new().span());
            let u_ident = Ident::new(&format!("fu_{}",&var_name), String::new().span());
            let var_name_s = var_name_lit.ident.to_string();
            let ident_type = var_name_lit.ident_type.clone();
            let val = var_name_lit.val.clone();
            let expanded = quote! {
            };
            let expanded_2 = quote! {
                if ident==#var_name_s {#u_ident(&val)}
            };
            
            (expanded,expanded_2)
        })
        .unzip();

    let expanded = quote!{
        pub fn goon_node_update(ident: &String, val: Vec<u8>) {
            #(
                #expanded_4
              )*
        }
        #(#expanded_2)*

        fn expand() {
            #(#expanded)*
        }
    };

    TokenStream::from(expanded)
}
#[derive(Clone)]
struct TypedMacroInput {
    ident: Ident,
    t1: Token![:],
    ident_type: Type,
    t2: Token![=],
    val: Expr,
}
impl syn::parse::Parse for TypedMacroInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            ident: input.parse()?,
            t1: input.parse()?,
            ident_type: input.parse()?,
            t2: input.parse()?,
            val: input.parse()?,
        })
    }
}
struct MacroInput {
    a: Expr,
    comma: Token![,],
    b: Expr,
}

impl syn::parse::Parse for MacroInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            a: input.parse()?,
            comma: input.parse()?,
            b: input.parse()?,
        })
    }
}

#[proc_macro_attribute]
pub fn goon_init(args: TokenStream, input: TokenStream) -> TokenStream {


    let args = parse_macro_input!(args as AttributeArgs);
//  let arg_value_get = match &args[..] {
//      [NestedMeta::Lit(Lit::Str(ref value))] => value.value(),
//      _ => panic!("The attribute should have exactly one string argument."),
//  };
    
    let input = parse_macro_input!(input as ItemFn);

    let fn_name = &input.sig.ident;
    let fn_inputs = &input.sig.inputs;
    let fn_body = &input.block;
    let data_struct = quote! {
        struct Server {
            socket: Global<std::net::UdpSocket>,
            peers: Global<std::collections::HashSet<u32>>,
            port: Global<u32>
        }
        impl Server {
            fn new(peers: Global<std::collections::HashSet<u32>>) -> Result<Server, std::io::Error> {
                Ok(Server { 
                    socket: Global::new(std::net::UdpSocket::bind("0.0.0.0:0")?),
                    peers,
                    port: Global::new(65535)
                })

            }

            fn start(&self) -> std::thread::JoinHandle<()>{
                let peers = self.peers.clone();
                let socket = self.socket.clone();
                let port = self.port.clone();
                std::thread::spawn(move || {
                    let mut lport = 65535;

                    while let Err(_) = std::net::UdpSocket::bind(&format!("0.0.0.0:{}", lport)){
                        lport -= 1;
                        if let Ok(mut peers) = peers.lock() {
                            peers.insert(lport);
                        };
                    }
                    let socket = if let Ok(mut socket) = socket.lock() {
                        if let Ok(mut port) = port.lock() {
                            *socket = std::net::UdpSocket::bind(&format!("0.0.0.0:{}", lport)).expect("howd this happen");
                            *port = lport;
                            Some(socket)
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    if let Some(socket) = socket {
                        // Buffer to store incoming data
                        let mut buf = vec![0u8; 8092];

                        loop {
                            // Receive data from the socket
                            let (amt, src) = socket.recv_from(&mut buf).expect("Failed to receive data");
                            let msg = std::str::from_utf8(&buf[..amt]).expect("failed to parse utf8 to string");
                            let packet: NodePacket = serde_json::from_str(msg).expect("failed to parse json");
                            match packet {
                                NodePacket::AddPeer(port) => {
                                    if let Ok(mut peers) = peers.lock() {
                                        peers.insert(port);
                                    };
                                },
                                NodePacket::Update((name, data)) => {
                                    goon_update!(name, data);
                                }
                            }
                        }
                    }
                })
            }
        }
        struct Client {
            socket: Global<std::net::UdpSocket>,
            peers: Global<std::collections::HashSet<u32>>
        }
        impl Client {
            fn new(peers: Global<std::collections::HashSet<u32>>) -> Result<Client, std::io::Error> {
                let socket = std::net::UdpSocket::bind("0.0.0.0:0")?;
                socket.set_broadcast(true)?;
                Ok(Client{ 
                    socket: Global::new(socket),
                    peers,
                })
            }

            fn send(&self, buf: &[u8]) {
                let peers = self.peers.clone();
                if let Ok(peers) = peers.lock() {
                    peers
                        .iter()
                        .for_each(|p| {
                            let address = format!("0.0.0.0:{}", p);
                            let socket = &self.socket;
                            read_globals!(socket;{
                                socket.send_to(buf, address);
                            });
                        });
                };
            }
        }
        struct Node {
            client: Client,
            server: Server,
            vars: std::collections::HashSet<String>
        }


        impl Node {
            fn new() -> Result<Node, std::io::Error> {
                let mut set: std::collections::HashSet<u32> = std::collections::HashSet::new();

                set.insert(65535);
                let peers = Global::new(set);
                let client = Client::new(peers.clone())?;
                let server = Server::new(peers)?;
                let vars = std::collections::HashSet::new();
                Ok(Node { client, server, vars})
            }

            fn start(&self) -> std::thread::JoinHandle<()> {
                self.server.start()
            }

            fn send(&self, msg: &[u8]) {
                self.client.send(msg);
            }

            fn get_port(&self) -> Option<u32> {
                if let Ok(port) = self.server.port.clone().lock() {
                    Some(*port)
                } else {
                    None
                }
            }

            fn contains(&self, ident: &String) -> bool {
                self.vars.contains(ident) 
            }

            fn add(&mut self, ident: &String, ) {
                self.vars.insert(ident.to_string());
            }

            fn update<T: Serialize>(&self, ident: String, data: T) {
                let ident = ident.to_uppercase();
                if let Ok(data) = serde_json::to_string(&data) {
                    if let Ok(data) = serde_json::to_string(&NodePacket::Update((ident.clone(),data.into_bytes()))) {
                        self.send(data.as_bytes());
                    }
                }
            }
        }


        #[derive(Serialize, Deserialize, Debug)]
        #[serde(crate = "self::serde")] // must be below the derive attribute
        enum NodePacket {
            AddPeer(u32),
            Update((String,Vec<u8>)),
        }
    };
    let start_code = quote! {
        expand();
        let NODE_HANDLER = if let Ok(node) = NODE.clone().lock() {
            let handler = node.start();

            std::thread::sleep(std::time::Duration::new(1, 0));
            if let Some(port) = node.get_port() {
                let packet: String = serde_json::to_string(&NodePacket::AddPeer(port))?;
                let packet = packet.into_bytes();

                node.send(&packet);
            }

            Some(handler)
        } else {
            None
        };
    };

    let end_code = quote! {
        if let Some(handler) = NODE_HANDLER {
            handler.join().expect("failed to join node handler");
        }
        Ok(())
    };
    let expanded = quote! {
        lazy_static::lazy_static! {
            static ref NODE: Global<Node> = Global::new(Node::new().expect("failed to create node context"));
        }
        #data_struct

        fn #fn_name() -> Result<(), Box<dyn std::error::Error + 'static>> {
            #start_code
            #fn_body
            #end_code
        }
    };
    expanded.into()
}

