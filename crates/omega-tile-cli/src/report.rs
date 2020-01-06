use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use omega_tile::Report;
use std::cell::RefCell;
use std::rc::Rc;
use texture_synthesis as ts;

pub struct SimpleProgressReport {
    ctx: Rc<RefCell<SimpleProgressContext>>,
}

impl SimpleProgressReport {
    pub fn new() -> SimpleProgressReport {
        SimpleProgressReport { ctx: Rc::new(RefCell::new(SimpleProgressContext::new())) }
    }
}

impl Report for SimpleProgressReport {
    fn sub_progress_bar(&mut self, title: &str) -> Box<dyn ts::GeneratorProgress> {
        let mut ctx = self.ctx.borrow_mut();
        ctx.stage_num = 0;
        ctx.total_pb.set_message(title);
        Box::new(SubProgressReport { ctx: self.ctx.clone() })
    }
}

struct SimpleProgressContext {
    total_pb: ProgressBar,
    stage_pb: ProgressBar,
    mthread: Option<std::thread::JoinHandle<()>>,

    total_len: usize,
    stage_len: usize,
    stage_num: u32,
}

impl SimpleProgressContext {
    pub(crate) fn new() -> SimpleProgressContext {
        let multi_pb = MultiProgress::new();
        let sty = ProgressStyle::default_bar()
            .template("[{msg:<20}] {bar:40.cyan/blue} {percent}%")
            .progress_chars("##-");

        let total_pb = multi_pb.add(ProgressBar::new(100));
        total_pb.set_style(sty);

        let sty = ProgressStyle::default_bar()
            .template(" stage {msg:<15} {bar:40.cyan/blue} {percent}%")
            .progress_chars("##-");
        let stage_pb = multi_pb.add(ProgressBar::new(100));
        stage_pb.set_style(sty);

        let mthread = std::thread::spawn(move || {
            let _ = multi_pb.join();
        });

        Self {
            total_pb,
            stage_pb,
            mthread: Some(mthread),
            total_len: 100,
            stage_len: 100,
            stage_num: 0,
        }
    }
}

pub struct SubProgressReport {
    ctx: Rc<RefCell<SimpleProgressContext>>,
}

impl Drop for SimpleProgressContext {
    fn drop(&mut self) {
        self.total_pb.finish();
        self.stage_pb.finish();
        self.mthread.take().map(|t| {
            let _ = t.join();
        });
    }
}

impl ts::GeneratorProgress for SubProgressReport {
    fn update(&mut self, update: ts::ProgressUpdate<'_>) {
        let mut ctx = self.ctx.borrow_mut();

        if update.total.total != ctx.total_len {
            ctx.total_len = update.total.total;
            ctx.total_pb.set_length(ctx.total_len as u64);
        }

        if update.stage.total != ctx.stage_len {
            ctx.stage_len = update.stage.total;
            ctx.stage_pb.set_length(ctx.stage_len as u64);
            ctx.stage_num += 1;
            ctx.stage_pb.set_message(&ctx.stage_num.to_string());
        }

        ctx.total_pb.set_position(update.total.current as u64);
        ctx.stage_pb.set_position(update.stage.current as u64);
    }
}
