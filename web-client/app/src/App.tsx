import { SiteView } from "./siteview";
import KeyManager from "./components/KeyManagerModal";

function App() {
  return (
    <div className="flex flex-col h-screen">
      <KeyManager />
      <SiteView page_route={window.location.pathname} />
    </div>
  );
}

export default App;
