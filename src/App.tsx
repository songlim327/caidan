import { useState, useEffect } from "react";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import "./App.css";

interface record {
  id: string;
  name: string;
  executable: string;
  logo: string;
}

function App() {
  const [appRecords, setAppRecords] = useState<record[]>([]);

  useEffect(() => {
    // probably include a loading animation for load config
    invoke<record[]>("read_config")
      .then(setAppRecords)
      .catch((error) => {
        console.error(error);
      });
  }, []);

  // handle click event for app records
  const click = (path: String) => {
    invoke("open_app", { path }).catch((error) => {
      console.error(error);
    });
  };

  return (
    <main id="container" className="box-border p-6 rounded-lg min-h-full w-full min-w-0">
      <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-6 justify-items-center gap-12">
        {appRecords.map((record: record) => (
          <div
            key={record.id}
            className="flex flex-col items-center justify-center p-4 pt-6 gap-4 cursor-pointer w-42 lg:w-48 rounded-md transition-colors duration-150 hover:bg-neutral-800"
            onClick={() => click(record.executable)}
            title={record.name}
          >
            <img src={convertFileSrc(record.logo)} width={48} height={48} />
            <h6 className="w-full truncate text-center">{record.name}</h6>
          </div>
        ))}
      </div>
    </main>
  );
}

export default App;
