Feature: Joypad keyboard mapping
  The emulator maps physical keyboard input to the NES joypad so games can
  be played. The joypad follows the NES hardware protocol (strobe + serial
  read of 8 buttons) and the keyboard mapping is configurable in code.

  Background:
    Given the emulator is running with a loaded ROM

  @functional
  Scenario: Strobe resets the button index
    Given joypad 1 is in use
    When the CPU writes 1 to address 0x4016
    Then the joypad button index is reset to 0
    And subsequent reads return button states starting from button A

  @functional
  Scenario: Serial read returns one button per read
    Given joypad 1 reports buttons A=pressed, B=released
    When the CPU reads address 0x4016 eight times
    Then the first read returns 1
    And the second read returns 0
    And reads 3 through 8 return 0
    And a ninth read returns 1

  @functional
  Scenario: Strobe low advances the button index
    Given joypad 1 reports button A pressed
    When the CPU writes 0 to address 0x4016
    And the CPU reads address 0x4016
    Then the returned value is 1
    And the next read returns the next button state

  @functional
  Scenario: Joypad 2 is independent
    Given joypad 1 reports A pressed
    And joypad 2 reports B pressed
    When the CPU reads 0x4016 and 0x4017
    Then 0x4016 returns the joypad 1 state
    And 0x4017 returns the joypad 2 state

  @functional
  Scenario: Keyboard press sets a joypad button
    Given the emulator window has focus
    When the user presses the key mapped to button A
    Then joypad 1's button A state becomes pressed
    And the next CPU read of 0x4016 reflects it

  @functional
  Scenario: Keyboard release clears a joypad button
    Given button A is currently pressed
    When the user releases the key mapped to button A
    Then joypad 1's button A state becomes released

  @functional
  Scenario: Multiple simultaneous presses
    Given the user presses A and Right at the same time
    When the CPU reads the joypad
    Then both A and Right are reported as pressed

  @edge-case
  Scenario: Unmapped key is ignored
    When the user presses a key not in the keymap
    Then no joypad button state changes

  @edge-case
  Scenario: Key events only when window focused
    Given the emulator window does not have focus
    When the user presses a mapped key
    Then no joypad button state changes

  @non-functional @observability
  Scenario: Input latency within frame budget
    When a key is pressed
    Then the corresponding joypad state is updated within one frame (16ms)
