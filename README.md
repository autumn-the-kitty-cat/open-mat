build with 
`cargo build --release`
then run
`openocd -f board/st_nucleo_f4.cfg -c "program target/thumbv7em-none-eabihf/release/rust-ili9341-init verify reset exit"`
