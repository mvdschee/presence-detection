fn main() {
	dotenvy::from_filename(".env").ok();

	// this allows envs at build time https://stackoverflow.com/questions/73041173/is-it-possible-to-use-env-file-at-build-time
	println!("cargo:rustc-env=WIFI_SSID={}", std::env::var("WIFI_SSID").unwrap());
	println!("cargo:rustc-env=WIFI_PASS={}", std::env::var("WIFI_PASS").unwrap());
	println!("cargo:rustc-env=MQTT_USERNAME={}", std::env::var("MQTT_USERNAME").unwrap());
	println!("cargo:rustc-env=MQTT_PASSWORD={}", std::env::var("MQTT_PASSWORD").unwrap());
	println!("cargo:rustc-env=MQTT_HOST={}", std::env::var("MQTT_HOST").unwrap());
	println!("cargo:rustc-env=MQTT_PORT={}", std::env::var("MQTT_PORT").unwrap());

	embuild::espidf::sysenv::output();
}
