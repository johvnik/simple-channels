use loom::thread;
use simple_channels::mpsc;

#[test]
fn loom_sends_and_receives_value() {
    loom::model(|| {
        let (tx, rx) = mpsc::bounded(4);

        thread::spawn(move || {
            tx.send("hello").unwrap();
        });

        let value = rx.recv().unwrap();
        assert_eq!(value, "hello");
    });
}
