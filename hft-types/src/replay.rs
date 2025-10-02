use crate::MarketTick;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

/// Market data recorder for backtesting
#[derive(Debug)]
pub struct MarketRecorder {
    file: File,
    tick_count: u64,
}

impl MarketRecorder {
    pub fn new<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let file = File::create(path)?;
        Ok(Self {
            file,
            tick_count: 0,
        })
    }

    pub fn record_tick(&mut self, tick: &MarketTick) -> std::io::Result<()> {
        let json = serde_json::to_string(tick)?;
        writeln!(self.file, "{}", json)?;
        self.tick_count += 1;
        Ok(())
    }

    pub fn tick_count(&self) -> u64 {
        self.tick_count
    }

    pub fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}

/// Market data replayer for backtesting
#[derive(Debug)]
pub struct MarketReplayer {
    reader: BufReader<File>,
    tick_count: u64,
}

impl MarketReplayer {
    pub fn new<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(Self {
            reader,
            tick_count: 0,
        })
    }

    pub fn next_tick(&mut self) -> std::io::Result<Option<MarketTick>> {
        let mut line = String::new();
        let bytes_read = self.reader.read_line(&mut line)?;

        if bytes_read == 0 {
            return Ok(None);
        }

        match serde_json::from_str(&line) {
            Ok(tick) => {
                self.tick_count += 1;
                Ok(Some(tick))
            }
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
        }
    }

    pub fn tick_count(&self) -> u64 {
        self.tick_count
    }
}

/// Replay statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct ReplayStats {
    pub total_ticks: u64,
    pub start_timestamp: u128,
    pub end_timestamp: u128,
    pub duration_ms: u64,
    pub symbols: Vec<String>,
}

impl ReplayStats {
    pub fn from_file<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let mut replayer = MarketReplayer::new(path)?;
        let mut total_ticks = 0u64;
        let mut start_timestamp = 0u128;
        let mut end_timestamp = 0u128;
        let mut symbols = std::collections::HashSet::new();

        while let Some(tick) = replayer.next_tick()? {
            if total_ticks == 0 {
                start_timestamp = tick.timestamp_nanos;
            }
            end_timestamp = tick.timestamp_nanos;
            symbols.insert(tick.symbol);
            total_ticks += 1;
        }

        let duration_ms = ((end_timestamp - start_timestamp) / 1_000_000) as u64;

        Ok(Self {
            total_ticks,
            start_timestamp,
            end_timestamp,
            duration_ms,
            symbols: symbols.into_iter().collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_record_and_replay() {
        let temp_file = "/tmp/hft_test_replay.jsonl";

        // Record some ticks
        {
            let mut recorder = MarketRecorder::new(temp_file).unwrap();
            for i in 0..10 {
                let tick = MarketTick::new(
                    "BTC/USD".to_string(),
                    45000.0 + i as f64,
                    100,
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_nanos(),
                );
                recorder.record_tick(&tick).unwrap();
            }
            recorder.flush().unwrap();
        }

        // Replay ticks
        {
            let mut replayer = MarketReplayer::new(temp_file).unwrap();
            let mut count = 0;
            while let Some(_tick) = replayer.next_tick().unwrap() {
                count += 1;
            }
            assert_eq!(count, 10);
        }

        // Get stats
        {
            let stats = ReplayStats::from_file(temp_file).unwrap();
            assert_eq!(stats.total_ticks, 10);
            assert!(stats.symbols.contains(&"BTC/USD".to_string()));
        }

        // Cleanup
        std::fs::remove_file(temp_file).unwrap();
    }
}
