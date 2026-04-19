#include <SPI.h>
#include <LoRa.h>

#define LORA_FREQ 915E6

void setup() {
  Serial.begin(9600); //start USB serial (used to send data to computer)
  delay(2000);  //give time for everything to start properly


  if (!LoRa.begin(LORA_FREQ)) {
    Serial.println("LoRa failed"); //if radio doesnt start
    while (1); //stop everything here
  }

}

void loop() {
      //check if a LoRa packet came in
  int packetSize = LoRa.parsePacket();
  if (!packetSize) return; //if nothing came in

  uint8_t buffer[512]; //store incoming fata
  int len = 0;//keep track of bytes we need


//reads the loRa data and stop if its the end of packet or buffer limit
  while (LoRa.available() && len < 512) {
    buffer[len++] = LoRa.read();
  }

//if nothing is read, exit
  if (len == 0) return;

  //send raw data to serial monitor

  Serial.write(buffer, len);
}