// MIDI interface for a morse paddle using a Digispark (ATtiny85)
//
// Dependencies:
// - DigiMIDI from https://github.com/heartscrytech/DigisparkMIDI
// - ezButton available in the Arduino library

#include <DigiMIDI.h>
#include <ezButton.h>

#define PIN_LED 1 // onboard LED

#define PIN_DIT 0
#define PIN_DAH 5

#define NOTE_DIT 1
#define NOTE_DAH 2

DigiMIDIDevice midi;

ezButton dit_button(PIN_DIT);
ezButton dah_button(PIN_DAH);

bool last_dit = false;
bool last_dah = false;

void setup() {
  pinMode(PIN_LED, OUTPUT);
  dit_button.setDebounceTime(5);
  dah_button.setDebounceTime(5);
}

void loop() {
  dit_button.loop();
  dah_button.loop();

  bool dit = !dit_button.getState();
  bool dah = !dah_button.getState();

  if (dit != last_dit) {
    if (dit)
      midi.sendNoteOn(NOTE_DIT, 1);
    else
      midi.sendNoteOff(NOTE_DIT, 0);
    last_dit = dit;
  }

  if (dah != last_dah) {
    if (dah)
      midi.sendNoteOn(NOTE_DAH, 1);
    else
      midi.sendNoteOff(NOTE_DAH, 0);
    last_dah = dah;
  }

  digitalWrite(PIN_LED, dit || dah ? HIGH : LOW);

  midi.update();
  midi.delay(5);
}
