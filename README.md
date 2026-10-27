# NMOS 6502 CPU-Core emulator library

This is a (mostly) cycle accurate NMOS 6502 emulator. The goal was to create a faithful recreation of the behavior original CPU.


## Testing

The tests rely on the SingleStepTests Test-Suite. It consists of a JSON file for each opcode. Since those files are very large, ca. 900MiB, they are not included in the repository. You can run the script `./setup_tests.sh` to download them and unpack them automatically. If you run the tests without the required JSON files, the tests will always fail.

Once downloaded, run `cargo test` to run all tests. You can also specify a specific opcode, e.g. by doing `cargo test 0xea`, which will run only the test for the opcode 0xea. You can also do `cargo test adc` to run all opcodes with the mnemonic ADC (add with carry).

Optionally, if the environment variable `TEST_6502_VERBOSE` is set, you can get more detailed output on which test failed and how. Make sure to only run one or two opcodes when doing this, otherwise you will get too much output to do anything with. You can run it in verbose mode by running `TEST_6502_VERBOSE=1 cargo test 0xea`.

## Known Issues

### unstable operations

Certain undocumented opcodes are unstable, meaning their behaviour is effectively non-deterministic. 
The result of such an operation can differ based on factors such as the chip's physical temperature, or slight manufacturing differences.
Those operations were implemented as NOPs, meaning they don't change the CPU State, but still advance the cycle count by the expected quantity.

The following are the unstable opcode Mnemonics:
- ANE
- SHA
- SHX
- SHY
- TAS

### ARR & SAX

The behaviour of ARR is deterministic according the refernce at masswerk.at, but I could not manage to make all tests pass. I have postponed fixing ARR for now, as I spent enough time without promising results.

SAX works in every addressing mode, except for Indirect,X. I have no idea why that might be, could not find any reason as to why through online research. The addressing mode works as expected, so either the tests are wrong, or that operation with that addressing mode behaves unpredictably or differently.

### Interrupt interface

Currently, there is no way to send either an IRQ or an NMI to the CPU.

## Resources

I've made use of a couple of online resources to faithfully recreate the behavior of the CPU, such as:

- [Masswerk 6502 Documentation](https://www.masswerk.at/6502/6502_instruction_set.html)
- [Decimal Mode Flag Behavior](https://forums.atariage.com/applications/core/interface/file/attachment.php?id=163231)
- [SingleStepTests Test-Suite](https://github.com/SingleStepTests/65x02/tree/main/6502)
