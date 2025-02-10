import { useState, useEffect, useRef } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./App.css";

interface MidiDevice {
  id: string;
  name: string;
}

interface Settings {
  default_midi_port: string | null;
}

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");
  const [midiDevices, setMidiDevices] = useState<MidiDevice[]>([]);
  const [selectedMidiPort, setSelectedMidiPort] = useState<string | null>(null);
  const [midiStatus, setMidiStatus] = useState<string>("");
  const [isRunning, setIsRunning] = useState(false);
  const canvasRef = useRef<HTMLCanvasElement>(null);

  type GamepadInput = [[number, number], [number, number]];

  useEffect(() => {
    const unlisten = listen<GamepadInput>("gamepad_input", (event) => {
      console.log("Gamepad Input: ", event.payload);
      const [leftStick, rightStick] = event.payload;
      drawSticks(leftStick, rightStick);
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, []);

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
        setMidiStatus(`Failed to load settings: ${error}`);
      }
    };
    loadInitialState();
  }, []);

  const handleMidiPortChange = (event: React.ChangeEvent<HTMLSelectElement>) => {
    setSelectedMidiPort(event.target.value);
  };

  function drawSticks(leftStick: [number, number], rightStick: [number, number]) {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    ctx.clearRect(0, 0, canvas.width, canvas.height);

    // Draw background circles
    ctx.beginPath();
    ctx.fillStyle = "lightgray";
    ctx.arc(canvas.width / 4, canvas.height / 2, canvas.height / 2, 0, 2 * Math.PI);
    ctx.fill();
    ctx.arc((3 * canvas.width) / 4, canvas.height / 2, canvas.height / 2, 0, 2 * Math.PI);
    ctx.fill();

    // Draw left stick
    ctx.beginPath();
    ctx.arc(
      canvas.width / 4 + (leftStick[0] * canvas.width) / 4,
      canvas.height / 2 - (leftStick[1] * canvas.height) / 2,
      10,
      0,
      2 * Math.PI
    );
    ctx.fillStyle = "red";
    ctx.fill();

    // Draw right stick
    ctx.beginPath();
    ctx.arc(
      (3 * canvas.width) / 4 + (rightStick[0] * canvas.width) / 4,
      canvas.height / 2 - (rightStick[1] * canvas.height) / 2,
      10,
      0,
      2 * Math.PI
    );
    ctx.fillStyle = "blue";
    ctx.fill();
  }

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name }));
  }

  async function startSystem() {
    if (!selectedMidiPort) {
      setMidiStatus("Please select a MIDI port first");
      return;
    }

    try {
      const result = await invoke<string>("start_system", { midiPort: selectedMidiPort });
      setMidiStatus(result);
      setIsRunning(true);
    } catch (error) {
      console.error("Failed to start system:", error);
      setMidiStatus(`Failed to start system: ${error}`);
      setIsRunning(false);
    }
  }

  async function stopSystem() {
    try {
      await invoke("stop_system");
      setMidiStatus("System stopped");
      setIsRunning(false);
    } catch (error) {
      console.error("Failed to stop system:", error);
      setMidiStatus(`Failed to stop system: ${error}`);
    }
  }

  return (
    <main className="container">
      <h1>Welcome to Tauri + React</h1>

      <div className="row">
        <a href="https://vitejs.dev" target="_blank">
          <img src="/vite.svg" className="logo vite" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" className="logo tauri" alt="Tauri logo" />
        </a>
        <a href="https://reactjs.org" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>

      <p>Click on the Tauri, Vite, and React logos to learn more.</p>

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
        <p>{midiStatus}</p>

        <button 
          onClick={isRunning ? stopSystem : startSystem}
          disabled={!selectedMidiPort && !isRunning}
        >
          {isRunning ? "Stop" : "Start"}
        </button>
      </div>

      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          greet();
        }}
      >
        <input
          id="greet-input"
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
        />
        <button type="submit">Greet</button>
      </form>
      <p>{greetMsg}</p>

      <canvas ref={canvasRef} width={600} height={300} style={{ border: "1px solid black" }}></canvas>
    </main>
  );
}

export default App;
