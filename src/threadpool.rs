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
    Job(Box<FnBox + Send + 'static>, usize),
    Terminate,
}

pub enum WorkerStatus {
    Complete(usize, usize),
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
                    Message::Job(job, job_id) => {
                        // println!("Worker {} received job {}; executing", id, job_id);
                        job.call();
                        match sender.send(WorkerStatus::Complete(id, job_id)) {
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

    pub fn execute<F>(&self, func: F, job_id: usize)
        where F: FnOnce() + Send + 'static {
        let job = Box::new(func);
        self.job_sender.send(Message::Job(job, job_id)).unwrap();
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
