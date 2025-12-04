import { useEffect, useState } from "react";
import { SiteView } from "./siteview";

function App() {
  let url = window.location.pathname.slice(1);
  if (url == "") {
    url = "zkpunks/postcatalogue";
  }
  const [route, setRoute] = useState(url);
  const [siteID, setSiteID] = useState("");

  //rewrite url on page navigation
  useEffect(() => {
    window.history.pushState({}, '', "/" + route);
  }, [route]);

  //handle back/forward page navigation
  useEffect(() => {
    const handlePopState = (_event: PopStateEvent) => {
      const path = window.location.pathname.slice(1);

      setRoute(path);
    };

    window.addEventListener('popstate', handlePopState);
    return () => {
      window.removeEventListener('popstate', handlePopState);
    };
  }, []);
  return (
    <div>

      <SiteView
        page_route={route}
        set_page_route={setRoute}
        site_id={siteID}
        set_site_id={setSiteID}
      ></SiteView>
    </div>
  );
}

export default App;
