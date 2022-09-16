use std::borrow::BorrowMut;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::thread::sleep;
use std::time::{Duration, SystemTime, SystemTimeError, UNIX_EPOCH};

pub type SequenceGeneratorSystemTimeError = SystemTimeError;

fn timestamp_from_custom_epoch(
    custom_epoch: SystemTime,
    micros_ten_power: u8,
) -> Result<u64, SequenceGeneratorSystemTimeError> {
    let timestamp;
    let mut micros_ten_power = micros_ten_power;
    if micros_ten_power >= 3 {
        timestamp = SystemTime::now().duration_since(custom_epoch)?.as_millis();
        micros_ten_power -= 3;
    } else {
        timestamp = SystemTime::now().duration_since(custom_epoch)?.as_micros();
    }
    match micros_ten_power {
        0 => Ok(timestamp as u64),
        _ => Ok((timestamp as u64) / (10_u64).pow(micros_ten_power.into())),
    }
}

#[derive(Debug)]
pub struct SequenceProperties {
    pub unused_bits: u8,
    pub timestamp_bits: u8,
    pub node_id_bits: u8,
    pub sequence_bits: u8,
    pub custom_epoch: SystemTime,
    current_timestamp: Rc<RefCell<Option<u64>>>,
    last_timestamp: Rc<RefCell<Option<u64>>>,
    pub micros_ten_power: u8,
    pub node_id: u16,
    pub sequence: Cell<u16>,
    pub max_sequence: u16,
    pub backoff_cooldown_start_ns: u64,
    partial_cached_id: Rc<RefCell<Option<u64>>>,
}

impl SequenceProperties {
    pub fn new(
        custom_epoch: SystemTime,
        node_id_bits: u8,
        node_id: u16,
        sequence_bits: u8,
        micros_ten_power: u8,
        unused_bits: u8,
        backoff_cooldown_start_ns: u64,
    ) -> Self {
        let timestamp_bits = (64_u8)
            .checked_sub(sequence_bits)
            .unwrap_or_else(|| {panic!(
                "Error: Sequence bits is too large '{}'", sequence_bits)})
            .checked_sub(node_id_bits)
            .unwrap_or_else(|| {panic!(
                "Error: Sum of bits is too large, maximum value 64. Node ID bits '{}', Sequence bits '{}'",
                node_id_bits, sequence_bits
            )})
            .checked_sub(unused_bits)
            .unwrap_or_else(|| {panic!(
                "Error: Sum of bits is too large, maximum value 64. Unused bits '{}', Sequence bits '{}', Node ID bits '{}'", 
                unused_bits, sequence_bits, node_id_bits
            )});
        SequenceProperties {
            custom_epoch,
            timestamp_bits,
            node_id_bits,
            sequence_bits,
            micros_ten_power,
            node_id,
            unused_bits,
            sequence: Cell::new(0),
            current_timestamp: Rc::new(RefCell::new(None)),
            last_timestamp: Rc::new(RefCell::new(None)),
            max_sequence: (2_u16).pow(sequence_bits.into()),
            backoff_cooldown_start_ns,
            partial_cached_id: Rc::new(RefCell::new(None)),
        }
    }
    pub fn set_last_timestamp(&self, timestamp: &mut Option<u64>) {
        if let Some(last_timestamp) = timestamp.take() {
            let _ = self
                .last_timestamp
                .as_ref()
                .borrow_mut()
                .insert(last_timestamp);
        }
    }
    pub fn set_current_timestamp(&self) {
        let _ = self.current_timestamp.as_ref().borrow_mut().insert(timestamp_from_custom_epoch(
            self.custom_epoch,
            self.micros_ten_power,
        ).unwrap_or_else(|error| {panic!("Error: Could not calculate current timestamp from custom epoch {:?} and micros power of {:?}. Error: {}",
        self.custom_epoch, self.micros_ten_power, error)}));
    }
    pub fn set_partial_cached_id(&self, cached_id: &mut Option<u64>) {
        let _ = self
            .partial_cached_id
            .as_ref()
            .borrow_mut()
            .insert(cached_id.take().unwrap());
    }
}

pub fn generate_id(
    properties: &SequenceProperties,
) -> Result<u64, SequenceGeneratorSystemTimeError> {
    properties.set_last_timestamp(&mut properties.current_timestamp.clone().take().take());
    properties.set_current_timestamp();
    if let Some(last_timestamp) = properties.last_timestamp.take() {
        let current_timestamp = properties.current_timestamp.borrow().unwrap();
        if current_timestamp < last_timestamp {
            println!("Error: System Clock moved backwards. Current timestamp '{}' is earlier than last registered '{}'.", 
                current_timestamp, last_timestamp);
            if properties.sequence.get() == properties.max_sequence {
                wait_next_timestamp(
                    last_timestamp,
                    properties.custom_epoch,
                    properties.micros_ten_power,
                    properties.backoff_cooldown_start_ns,
                )?;
                // After timestamp changed reset to start a new sequence
                properties.sequence.set(0);
            } else {
                wait_until_last_timestamp(
                    last_timestamp,
                    properties.custom_epoch,
                    properties.micros_ten_power,
                    properties.backoff_cooldown_start_ns,
                )?;
            }
            properties.set_current_timestamp();
        } else if properties.current_timestamp.borrow().unwrap() != last_timestamp {
            properties.sequence.set(0);
        }
    }
    let new_id = to_id(properties);
    properties.sequence.set(properties.sequence.get() + 1);
    if properties.sequence.get() == properties.max_sequence {
        wait_next_timestamp(
            properties.current_timestamp.borrow().unwrap(),
            properties.custom_epoch,
            properties.micros_ten_power,
            properties.backoff_cooldown_start_ns,
        )?;
        properties.set_current_timestamp();
        // After timestamp changed reset to start a new sequence
        properties.sequence.set(0);
    }
    Ok(new_id)
}

fn wait_next_timestamp(
    last_timestamp: u64,
    custom_epoch: SystemTime,
    micros_ten_power: u8,
    backoff_cooldown_start_ns: u64,
) -> Result<(), SequenceGeneratorSystemTimeError> {
    let mut current_timestamp = timestamp_from_custom_epoch(custom_epoch, micros_ten_power)?;
    let backoff_cooldown_ns: u64 = backoff_cooldown_start_ns;
    while current_timestamp <= last_timestamp {
        sleep(Duration::from_nanos(backoff_cooldown_ns));
        current_timestamp = timestamp_from_custom_epoch(custom_epoch, micros_ten_power)?;
        // Double the cooldown wait period (exponential backoff)
        backoff_cooldown_ns
            .checked_add(backoff_cooldown_ns)
            .unwrap_or_else(|| {
                panic!(
                    "Error: Cannot double backoff cooldown, maximum value reached '{}'",
                    backoff_cooldown_ns
                )
            });
    }
    Ok(())
}

fn wait_until_last_timestamp(
    last_timestamp: u64,
    custom_epoch: SystemTime,
    micros_ten_power: u8,
    backoff_cooldown_start_ns: u64,
) -> Result<(), SequenceGeneratorSystemTimeError> {
    let mut current_timestamp = timestamp_from_custom_epoch(custom_epoch, micros_ten_power)?;
    let backoff_cooldown_ns: u64 = backoff_cooldown_start_ns;
    while current_timestamp < last_timestamp {
        sleep(Duration::from_nanos(backoff_cooldown_ns));
        current_timestamp = timestamp_from_custom_epoch(custom_epoch, micros_ten_power)?;
        // Double the cooldown wait period (exponential backoff)
        backoff_cooldown_ns
            .checked_add(backoff_cooldown_ns)
            .unwrap_or_else(|| {
                panic!(
                    "Error: Cannot double backoff cooldown, maximum value reached '{}'",
                    backoff_cooldown_ns
                )
            });
    }
    Ok(())
}

fn to_id_cached(properties: &SequenceProperties) -> u64 {
    let mut id = properties.partial_cached_id.as_ref().borrow().unwrap();
    id |= ((properties.sequence.get() as u64) << properties.node_id_bits) as u64;
    id
}

fn to_id(properties: &SequenceProperties) -> u64 {
    if properties.sequence.get() == 0 {
        cache_partial_id(properties);
    }
    to_id_cached(properties)
}

fn cache_partial_id(properties: &SequenceProperties) {
    let timestamp_shift_bits = properties.node_id_bits + properties.sequence_bits;
    let mut id = properties.current_timestamp.borrow().unwrap() << timestamp_shift_bits;
    id |= properties.node_id as u64;
    properties.set_partial_cached_id(Some(id).borrow_mut());
}

pub fn decode_id_unix_epoch_micros(id: u64, properties: &SequenceProperties) -> u64 {
    let id_timestamp_custom_epoch = (id << (properties.unused_bits))
        >> (properties.node_id_bits + properties.sequence_bits + properties.unused_bits);
    let timestamp_micros =
        id_timestamp_custom_epoch * (10_u64).pow(properties.micros_ten_power as u32);
    properties
        .custom_epoch
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| {panic!("Error: Could not calculate difference between timestamp decoded from ID and Unix epoch.")})
        .checked_add(Duration::from_micros(timestamp_micros))
        .unwrap_or_else(|| {panic!("Error: Could not add the timestamp decoded from ID to the provided custom epoch.")})
        .as_micros() as u64
}

pub fn decode_node_id(id: u64, properties: &SequenceProperties) -> u16 {
    ((id << (properties.unused_bits + properties.timestamp_bits + properties.sequence_bits))
        >> (properties.sequence_bits + properties.timestamp_bits + properties.unused_bits))
        as u16
}

pub fn decode_sequence_id(id: u64, properties: &SequenceProperties) -> u16 {
    ((id << (properties.unused_bits + properties.timestamp_bits))
        >> (properties.unused_bits + properties.timestamp_bits + properties.node_id_bits))
        as u16
}

#[cfg(test)]
mod tests {
    #[test]
    fn timestamp_from() {
        // Perform consistency tests for datetime calculation from a custom epoch
        // First case: Compare system time against custom epoch set to UNIX_EPOCH
        // Second case: Set CUSTOM_EPOCH to test start time and compare timestamp
        // calculation against known sleep duration interval
        use super::*;
        let time_now = SystemTime::now();
        let millis_start = time_now
            .duration_since(UNIX_EPOCH)
            .expect("Error: Failed to get current time as duration from epoch.")
            .as_millis();
        sleep(Duration::from_millis(50));
        // Test UNIX EPOCH
        let millis_after = timestamp_from_custom_epoch(UNIX_EPOCH, 3).unwrap_or_else(
            |error| {
                panic!(
                    "SequenceGeneratorSystemTimeError: Failed to get timestamp from custom epoch {:?}, difference {:?}",
                    UNIX_EPOCH, (error).duration()
                )
            });
        // More than expected 50ms. Upper boundary cannot be ascertained as Normal distribution
        // CPU low-power states and/or older hardware can cause signifficant differences.
        // (although rather then a Normal distribution, it is instead the case that a Pareto
        // distribution applies, making it impossible to set high enough value for the test
        // not to fail on ocassion)
        let substracted_times = millis_after.checked_sub(millis_start as u64).unwrap();
        println!("Too small time difference between times calculated\nfrom UNIX_EPOCH using independent functions.\n\nEpoch System Time - Time Difference w/Epoch = {} ms,\nexpected greater or equals than sleep interval 50 ms.\n", substracted_times);
        assert!(substracted_times >= 50);
        // If too big upper boundary there could be numerical errors.
        assert!((millis_after.checked_sub(millis_start as u64).unwrap()) < 90);
        // Test a CUSTOM EPOCH in tenths of a millisecond
        let custom_epoch = UNIX_EPOCH
            .checked_add(Duration::from_millis(millis_start as u64))
            .expect("Error: Failed to create custom epoch.");
        let tenths_millis_custom_epoch_time = timestamp_from_custom_epoch(custom_epoch, 2).unwrap_or_else(
            |error| {
                panic!(
                    "SequenceGeneratorSystemTimeError: Failed to get current timestamp from custom epoch {:?}, difference {:?}",
                    UNIX_EPOCH, (error).duration()
                )
            });
        // Wait a bit to prevent Option to call unwrap() on None below
        // If both timestamps are within small margin substraction of u64
        // can result in 'panicked at attempt to subtract with overflow'
        // and checked_sub returns None value
        sleep(Duration::from_millis(2));
        // convert elapsed time from microseconds into tenths of a millisecond (0,1ms = 100 mcs)
        let power_two: u32 = 2;
        let tenths_millis_elapsed_time = (time_now.elapsed().map_or_else(
            |error| {
                panic!(
                    "SequenceGeneratorSystemTimeError: Failed to get elapsed time, difference {:?}",
                    (error).duration()
                )
            },
            |duration| duration.as_micros() as u64,
        )) / (10_u64).pow(power_two);
        let substracted_times = tenths_millis_elapsed_time
            .checked_sub(tenths_millis_custom_epoch_time)
            .unwrap();
        println!("Too high time difference between calculated time from\nCustom Epoch set at test start and actual elapsed\ntime since the test started.\n\nElapsed Time - Calculated Time Custom Epoch = {} mcs,\nexpected under 100 mcs\n\nPlease note that Pareto distribution applies and it\nis impossible to ensure a high enough difference for\nthe test not to fail on ocassion.\n\nReview only after ensuring repeated failures.\n", substracted_times);
        // Substract custom epoch result with Rust's own elapsed time
        // Upper boundary uncertainty set up high at 200mcs more than expected 511mcs as exponential
        // distribution, CPU low-power states and/or older hardware can cause signifficant differences.
        assert!(substracted_times < 200);
    }

    #[test]
    fn wait_until() {
        // Case where system clock is readjusted 50ms into the past
        // Current sequence wouldn't be exhausted but script cools down
        // until at least matching the previously stored timestamp.
        use super::*;
        let calculated_time_after_50ms: u64 = SystemTime::now()
            .checked_add(Duration::from_millis(50))
            .unwrap()
            .duration_since(UNIX_EPOCH)
            .expect("Error: Failed to get duration from epoch of timestamp 50ms into the future.")
            .as_millis() as u64;
        // Function itself serves as an sleep call if correct
        wait_until_last_timestamp(calculated_time_after_50ms, UNIX_EPOCH, 3, 1500).expect(
            &format!(
            "SequenceGeneratorSystemTimeError: Couldn't wait until timestamp '{}' with custom epoch '{:?}'",
            calculated_time_after_50ms, UNIX_EPOCH
        ),
        );
        // Wait a bit to prevent Option to call unwrap() on None below
        // If both timestamps are within small margin substraction of u64
        // can result in 'panicked at attempt to subtract with overflow'
        // and checked_sub returns None value.
        // Furthermore: It could also result in useless assert comparing
        // if an unsigned integer is higher or equal to zero
        sleep(Duration::from_millis(1));
        let time_after_50ms: u64 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Error: Failed to get current time as duration from epoch.")
            .as_millis() as u64;
        let substracted_times = time_after_50ms
            .checked_sub(calculated_time_after_50ms)
            .unwrap();
        assert!(substracted_times > 0);
        println!("Too high time difference while waiting for last timestamp\nafter clock moved backwards\n\nTime Calculated - Actual Time = {} ms, expected under 35 ms\n\nPlease note that Pareto distribution applies and it\nis impossible to ensure a high enough difference for\nthe test not to fail on ocassion.\n\nReview only after ensuring repeated failures.\n", substracted_times);
        // Assert an upper boundary to how high of a difference there can be.
        // If implementation is correct, the timestampts should be within few
        // ms of one another according to a Normal distribution in recent
        // hardware and normal CPU priority (although rather a Pareto
        // distribution applies, making it impossible to set a value high
        // enough for the test not to fail on ocassion)
        assert!(substracted_times < 35);
    }
    #[test]
    fn wait_next() {
        // Case where sequence would be exhausted and for that reason
        // script cools down until at least there exists a difference
        // between the current system time and the last known timestamp.
        use super::*;
        let calculated_time_after_10ms: u64 = SystemTime::now()
            .checked_add(Duration::from_millis(10))
            .expect("Error: Failed to 10ms to current timestamp.")
            .duration_since(UNIX_EPOCH)
            .expect("Error: Failed to get duration from epoch of timestamp 10ms into the future.")
            .as_millis() as u64;
        // Function itself serves as an sleep call if correct
        wait_next_timestamp(calculated_time_after_10ms, UNIX_EPOCH, 3, 1500).unwrap_or_else(|_| {panic!(
            "SequenceGeneratorSystemTimeError: Couldn't wait until timestamp '{}' with custom epoch '{:?}'",
            calculated_time_after_10ms, UNIX_EPOCH
        )});
        let time_after_11ms: u64 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Error: Failed to get current time as duration from epoch.")
            .as_millis() as u64;
        let substracted_times = time_after_11ms
            .checked_sub(calculated_time_after_10ms)
            .unwrap();
        assert!(substracted_times > 0);
        println!("Too high time difference while waiting for next timestamp\n\nNext timestamp - Last Timestamp = {} ms, expected under 35 ms\n\nPlease note that Pareto distribution applies and it\nis impossible to ensure a high enough difference for\nthe test not to fail on ocassion.\n\nReview only after ensuring repeated failures.\n", substracted_times);
        // Assert an upper boundary to how high of a difference there can be.
        // If implementation is correct, the timestampts should be within few
        // ms of one another according to a Normal distribution in recent
        // hardware and normal CPU priority (although rather a Pareto
        // distribution applies, making it impossible to set a value high
        // enough for the test not to fail on ocassion)
        assert!(substracted_times < 35);
    }
    #[test]
    fn gen_id() {
        use super::*;
        use rand::Rng;
        // timestamp with 39 bits
        let custom_epoch = UNIX_EPOCH;
        // 2^16 node id (up to 65536)
        let node_id_bits = 16;
        // Several unused bits
        let unused_bits = 7;
        // 2^2 sequence (up to 4)
        // To test sequence overflow and wait behaviour, sequence bits unrealistically low
        let sequence_bits = 2;
        // in centiseconds (10^4 mcs)
        let micros_ten_power = 4;
        let mut rng = rand::thread_rng();
        // 0..2^16-1
        let node_id = rng.gen_range(0..65535);
        // if stalled until next millisecond, begin exponential backoff at 1,5 mcs
        let backoff_cooldown_start_ns = 1_000_000;
        let last_timestamp = (SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Error: Failed to get current time as duration from epoch.")
            .as_millis()
            / 10) as u64;
        // Ensure a new fresh second
        wait_next_timestamp(
            last_timestamp,
            UNIX_EPOCH,
            micros_ten_power,
            backoff_cooldown_start_ns,
        )
        .unwrap_or_else(|_| {panic!(
            "SequenceGeneratorSystemTimeError: Couldn't wait until timestamp '{}' with custom epoch '{:?}'",
            last_timestamp, UNIX_EPOCH
        )});
        let mut vector_ids: Vec<u64> = vec![0; 5];
        let properties = SequenceProperties::new(
            custom_epoch,
            node_id_bits,
            node_id,
            sequence_bits,
            micros_ten_power,
            unused_bits,
            backoff_cooldown_start_ns,
        );
        for element in vector_ids.iter_mut() {
            *element = generate_id(&properties).unwrap_or_else(
                |error| {
                    panic!(
                        "SequenceGeneratorSystemTimeError: Failed to get timestamp from custom epoch {:?}, difference {:?}",
                        UNIX_EPOCH, (error).duration()
                    )
                });
        }
        let decoded_timestamp = decode_id_unix_epoch_micros(vector_ids[0], &properties);
        assert!(((decoded_timestamp / 10_000) - (last_timestamp + 1)) < 15);
        let mut decoded_node_id = decode_node_id(vector_ids[0], &properties);
        assert_eq!(decoded_node_id, node_id);
        let mut decoded_seq_id = decode_sequence_id(vector_ids[0], &properties);
        assert_eq!(decoded_seq_id, 0);
        for index in 1..5 {
            decoded_seq_id = decode_sequence_id(vector_ids[index], &properties);
            assert_eq!(decoded_seq_id, (index as u16) % 4);
            decoded_node_id = decode_node_id(vector_ids[index], &properties);
            assert_eq!(decoded_node_id, node_id);
        }
        assert!(properties.current_timestamp.borrow().unwrap() - last_timestamp < 15);
    }
}
