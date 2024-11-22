import { useEffect, useState } from "react";
import "./App.css";

import { SerialPort } from "tauri-plugin-serialplugin";
import * as os from "@tauri-apps/plugin-os";

import { Slider } from "@material-tailwind/react";

function App() {
  const [thumbServoDegree, setThumbServoDegree] = useState(50);
  const [armServoDegree, setArmServoDegree] = useState(50);
  const [fingersServoDegree, setFingersServoDegree] = useState(50);

  const [serialPort, setSerialPort] = useState<SerialPort | undefined>();

  const connect = async () => {
    await SerialPort.closeAll();
    const ports = await SerialPort.available_ports();

    for (const [name, info] of Object.entries(ports)) {
      if (info.manufacturer != 'Devils Prosthetics' && info.serial_number.toUpperCase() !== 'DEVIL') continue;

      if (os.platform() == "macos" && name.includes('tty')) continue;

      console.log(`connecting to ${name}`)

      const serialPort = new SerialPort({
        path: name,
        baudRate: 115200
      });

      await serialPort.open();

      setSerialPort(serialPort);

      break;
    }
  }

  useEffect(() => {
    if (!serialPort) {
      connect();
    }
  }, [serialPort]);

  const sendDegree = async (event: React.ChangeEvent<HTMLInputElement>, servo: 'thumb' | 'arm' | 'fingers') => {
    const degree = Math.round(Number(event.target.value));

    switch (servo) {
      case "thumb":
        setThumbServoDegree(degree);
        break;
      case "arm":
        setArmServoDegree(degree);
        break;
      case "fingers":
        setFingersServoDegree(degree);
        break;
    }

    const command = `${servo} ${degree}`;

    console.log(Uint8Array.from(command.split('').map(letter => letter.charCodeAt(0))));

    if (serialPort) {
      serialPort.write(command);
    } 
  }

  return (
    <main className="flex flex-col justify-center items-center h-full w-full px-10 bg-gray-900 text-gray-200">
      <h1 className="text-3xl">Servo Testing Software</h1>
      <p className="mb-2 text-gray-300">
        &emsp;&emsp;Just drag the slider with the Raspberry Pi Pico plugged in, and this will automatically set the
        corresponding servo to that degree from 0-180. Thumb servo is expected to be plugged into gpio pin 2,
        arm servo is expected to be plugged into gpio pin 3, and the fingers servo is expected to be in gpio pin 4.
        The servo has three colors, red is positive, black is negative, white is signal, which gets plugged 
        into those pins
      </p>

      <button className="mb-10 bg-white text-black p-2 rounded-xl hover:bg-gray-200" onClick={() => connect()}>reconnect</button>

      <h2 className="text-2xl mb-2">Thumb Servo</h2>
      {/* @ts-ignore */}
      <Slider className="mb-7" color="blue" size="lg" value={thumbServoDegree} onChange={event => sendDegree(event, 'thumb')} />

      <h2 className="text-2xl mb-2">Arm Servo</h2>
      {/* @ts-ignore */}
      <Slider className="mb-7" color="red" size="lg" value={armServoDegree} onChange={event => sendDegree(event, 'arm')} />

      <h2 className="text-2xl mb-2">Fingers Servo</h2>
      {/* @ts-ignore */}
      <Slider className="mb-7" color="green" size="lg" value={fingersServoDegree} onChange={event => sendDegree(event, 'fingers')} />

    </main>
  );
}

export default App;
