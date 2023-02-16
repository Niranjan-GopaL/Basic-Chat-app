//importing rocket globally using the macro_use attribute
//so that rocket macros can be used anywhere in the application
#[macro_use] extern crate rocket;


use rocket::{State, Shutdown};
use rocket::fs::{relative, FileServer};
use rocket::form::Form;
use rocket::response::stream::{EventStream, Event};
use rocket::serde::{Serialize, Deserialize};
use rocket::tokio::sync::broadcast::{channel, Sender, error::RecvError};
use rocket::tokio::select;

/* 
//this is a route called world.
//route attribute describes the type of request 
//this route handles in this route
//get request to the path
            #[get("/world")]
//handler function desrubes how that request is to be processed
//here the handler has no parameters and returns a string 
        fn world() -> &'static str {
            "Hello world!"
        }
*/

// deriving few TRAITS
#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]


//struct has 3 fields , extra validation added to username and roomname
struct Message{
    #[field(validate = len(..30))]
    pub room: String,
    #[field(validate = len(..20))]
    pub username: String,
    pub message: String,
}

// After implementing message , final job: impleement your end points
//only 2 end points are needed : one endpoint to post messages another to receive messages 

// this route matches against the post request to the message path and accepts form data
#[post("/message", data = "<form>")]
// function handler takes in 2 parameters :form data and server state
fn post(form: Form<Message>, queue:&State<Sender<Message>>)  {
    //A send fails if their are no active subcribers (recievers) , but that's fine.
    // here we simply send message to all recievers
    let _res = queue.send(form.into_inner());
}


/// Returns an infinite stream of server-sent events. Server-sent events allow clients to 
/// open a long lived connection with server adn then servver can send data to the client whenever it wants.
/// Similat to web sockets except it works only in one dir
/// Each event is a message pulled from a broadcast queue sent by the `post` handler.

// takes to param : queue is the server state and end is of type Shutdown
#[get("/events")]
async fn events(queue: &State<Sender<Message>>, mut end: Shutdown) -> EventStream![] {
    // creating a new reciever
    let mut rx = queue.subscribe();
    EventStream! {
        loop {
            // selects waits on multiple concurrent branches and returns as soon as one of them completes
            let msg = select! {
                msg = rx.recv() => match msg {
                    Ok(msg) => msg,
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(_)) => continue,
                },
                _ = &mut end => break,
            };
// assuming we don't hit one of these break or continue statements , the select macro will return the message we got from
//  our receiver at which point we can yield a new server sent event passing in our message
            yield Event::json(&msg);
        }
    }
}


//this is launch attribute generates a main fn
// manage methode allows use to add STATE to our rocket server instance which all handlers will have access to
//state we wanna add is a channel or (sender end of a channel)
//tokio as async runtime
//channels are differenct way to pass messages between different async tasks

//tokio channels , async programming LEARN THESE TO UNDERSTAND  

//capacity : amount of messages that can be in retained in a channel at a time 
// channel function returns a tuple - containing a sender end and a receiver end

// Last thing to o is mounting the routes - post and events
// mounting a handler that will serve static files 
#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(channel::<Message>(1024).0) 
        .mount("/" , routes![post, events])
        .mount("/", FileServer::from(relative!("static")))
}


//the message endpoint will receive new messages and send them down
//the channel and the events endpoint listens to messages coming down the channel and sends them to clients
// https://github.com/Niranjan-GopaL/Basic-Chat-app.git
