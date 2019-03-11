use std::thread;
use std::sync::{mpsc, Arc, Mutex};


trait FnBox {
    fn call(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call(self: Box<Self>) {
        return (*self)();
    }
}

enum Message {
    Job(ExecNode, usize),
    Terminate,
}

pub enum WorkerStatus {
    Complete(usize, ExecNode),
}

pub struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, job_receiver: Arc<Mutex<mpsc::Receiver<Message>>>, sender: mpsc::Sender<WorkerStatus>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let message = job_receiver.lock().unwrap().recv().unwrap();
                match message {
                    Message::Job(mut job, job_id) => {
                        // println!("Worker {} received job {}; executing", id, job_id);
                        job.execute();
                        match sender.send(WorkerStatus::Complete(id, job)) {
                            Ok(_) => (),
                            Err(what) => panic!("Worker {} could not send job {} status.\n{}", id, job_id, what),
                        };
                        // println!("Worker {} finished executing job {}", id, job_id);
                    },
                    Message::Terminate => {
                        // println!("Worker {} shutting down", id);
                        break;
                    }
                }


            }
        });
        return Worker{id: id, thread: Some(thread)};
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    job_sender: mpsc::Sender<Message>,
    pub wstatus_receiver: mpsc::Receiver<WorkerStatus>,
}

#[derive(Debug)]
pub struct ExecNode {x: usize}

impl ExecNode {
    pub fn new() -> ExecNode {
        return ExecNode{x: 1};
    }

    fn execute(&mut self) {
        self.x = 0;
    }
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let (job_sender, job_receiver) = mpsc::channel();
        let job_receiver = Arc::new(Mutex::new(job_receiver));

        let (wstatus_sender, wstatus_receiver) = mpsc::channel();

        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&job_receiver), wstatus_sender.clone()));
        }

        return ThreadPool{workers: workers, job_sender: job_sender, wstatus_receiver: wstatus_receiver};
    }

    pub fn execute(&self, node: ExecNode, job_id: usize) {
        // self.job_sender.send(Message::Job(Arc::clone(&node), job_id)).unwrap();
        self.job_sender.send(Message::Job(node, job_id)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in &self.workers {
            self.job_sender.send(Message::Terminate).unwrap();
        }

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
