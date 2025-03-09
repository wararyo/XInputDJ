var XInputDJ = function() {
    var RESOLUTION = 360;
    var RECORD_SPEED = 33 + (1 / 3);
    var ALPHA = 1.0 / 8;
    var BETA = ALPHA / 32;

    this.scratch = function(channel, control, value, status, group) {
        if ((status & 0xF0) === 0x90) {
            engine.scratchEnable(script.deckFromGroup(group), RESOLUTION, RECORD_SPEED, ALPHA, BETA);
        } else {
            engine.scratchDisable(script.deckFromGroup(group));
        }
    };

    this.wheelTurn = function(channel, control, value, status, group) {
        var newValue = (value < 64) ? value : value - 128;
        if (engine.isScratching(script.deckFromGroup(group))) {
            engine.scratchTick(script.deckFromGroup(group), newValue); // Scratch!
        } else {
            engine.setValue(group, 'jog', newValue); // Pitch bend
        }
    };
};

XInputDJ = new XInputDJ();
