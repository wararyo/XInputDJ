import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface MidiDevice {
  id: string;
  name: string;
}

interface Settings {
  default_midi_port: string | null;
}

function App() {
  const [midiDevices, setMidiDevices] = useState<MidiDevice[]>([]);
  const [selectedMidiPort, setSelectedMidiPort] = useState<string | null>(null);
  const [statusMessage, setStatusMessage] = useState<string>("");
  const [isRunning, setIsRunning] = useState(false);

  useEffect(() => {
    const loadInitialState = async () => {
      try {
        const devices = await invoke<MidiDevice[]>("get_midi_ports");
        setMidiDevices(devices);

        const settings = await invoke<Settings>("get_settings");
        if (settings.default_midi_port) {
          setSelectedMidiPort(settings.default_midi_port);
        }
      } catch (error) {
        console.error("Failed to load initial state:", error);
        setStatusMessage(`Failed to load settings: ${error}`);
      }
    };
    loadInitialState();
  }, []);

  const handleMidiPortChange = (event: React.ChangeEvent<HTMLSelectElement>) => {
    setSelectedMidiPort(event.target.value);
  };

  async function startSystem() {
    if (!selectedMidiPort) {
      setStatusMessage("Please select a MIDI port first");
      return;
    }

    try {
      const result = await invoke<string>("start_system", { midiPort: selectedMidiPort });
      setStatusMessage(result);
      setIsRunning(true);
    } catch (error) {
      console.error("Failed to start system:", error);
      setStatusMessage(`Failed to start system: ${error}`);
      setIsRunning(false);
    }
  }

  async function stopSystem() {
    try {
      await invoke("stop_system");
      setStatusMessage("System stopped");
      setIsRunning(false);
    } catch (error) {
      console.error("Failed to stop system:", error);
      setStatusMessage(`Failed to stop system: ${error}`);
    }
  }

  return (
    <main className="container">
      <div className="midi-controls">
        <select 
          value={selectedMidiPort !== null ? selectedMidiPort : ""} 
          onChange={handleMidiPortChange}
          disabled={isRunning}
        >
          <option value="">Select MIDI Output</option>
          {midiDevices.map((device) => (
            <option key={device.id} value={device.id}>
              {device.name}
            </option>
          ))}
        </select>
        <p>{statusMessage}</p>

        <button 
          onClick={isRunning ? stopSystem : startSystem}
          disabled={!selectedMidiPort && !isRunning}
        >
          {isRunning ? "Stop" : "Start"}
        </button>
      </div>
    </main>
  );
}

export default App;
