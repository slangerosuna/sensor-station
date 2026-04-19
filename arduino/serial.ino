#include <SPI.h>
#include <LoRa.h>

#define LORA_FREQ 915E6
#define MAX_PACKET_SIZE 2048

uint8_t chunkBuffer[256][253];
uint8_t chunkLengths[256];
bool chunkReceived[256];
uint8_t totalChunksExpected = 0;
uint8_t chunksReceived = 0;

void resetChunkState() {
  memset(chunkReceived, 0, sizeof(chunkReceived));
  memset(chunkLengths, 0, sizeof(chunkLengths));
  totalChunksExpected = 0;
  chunksReceived = 0;
}

void setup() {
  Serial.begin(115200);
  delay(2000);

  if (!LoRa.begin(LORA_FREQ)) {
    Serial.println("LoRa failed");
    while (1);
  }
  resetChunkState();
}

void loop() {
  int packetSize = LoRa.parsePacket();
  if (!packetSize) return;

  uint8_t raw[255];
  int len = 0;
  while (LoRa.available() && len < 255) {
    raw[len++] = LoRa.read();
  }

  if (len < 2) return;

  uint8_t chunkIndex  = raw[0];
  uint8_t totalChunks = raw[1];
  uint8_t* payload    = raw + 2;
  uint8_t payloadLen  = len - 2;

  if (chunksReceived == 0) {
    totalChunksExpected = totalChunks;
  }

  if (totalChunks != totalChunksExpected) {
    resetChunkState();
    totalChunksExpected = totalChunks;
  }

  if (!chunkReceived[chunkIndex]) {
    memcpy(chunkBuffer[chunkIndex], payload, payloadLen);
    chunkLengths[chunkIndex] = payloadLen;
    chunkReceived[chunkIndex] = true;
    chunksReceived++;
  }

  if (chunksReceived < totalChunksExpected) return;

  // Reassemble full packet
  uint8_t fullPacket[MAX_PACKET_SIZE];
  int fullLen = 0;

  for (int i = 0; i < totalChunksExpected; i++) {
    if (!chunkReceived[i]) {
      resetChunkState();
      return;
    }
    if (fullLen + chunkLengths[i] > MAX_PACKET_SIZE) {
      resetChunkState();
      return;
    }
    memcpy(fullPacket + fullLen, chunkBuffer[i], chunkLengths[i]);
    fullLen += chunkLengths[i];
  }

  // Validate 4-byte packet header
  if (fullLen < 4) {
    resetChunkState();
    return;
  }

  uint16_t msgLength = (uint16_t)fullPacket[2] | ((uint16_t)fullPacket[3] << 8);

  if (msgLength != fullLen - 4) {
    resetChunkState();
    return;
  }

  // Send to Python: 2-byte length prefix + full packet
  Serial.write((uint8_t*)&fullLen, 2);
  Serial.write(fullPacket, fullLen);

  resetChunkState();
}