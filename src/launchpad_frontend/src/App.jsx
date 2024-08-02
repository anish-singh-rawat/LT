import { useState } from 'react';
import { launchpad_backend } from 'declarations/launchpad_backend';

function App() {
  const [greeting, setGreeting] = useState('');

  const handleSubmit = async (event) =>{
    event.preventDefault();
    const name = event.target.elements.name.value;
    const payload ={
      id : Math.floor(Math.random() * 10000),
      name : name
    }
    const greeting = await launchpad_backend.add_message(payload)
      setGreeting(greeting);
      console.log(greeting,'greeting')
  }
  
  return (
    <main>
      <img src="/logo2.svg" alt="DFINITY logo" />
      <br />
      <br />
      <form action="#" onSubmit={handleSubmit}>
        <label htmlFor="name">Enter your name: &nbsp;</label>
        <input id="name" alt="Name" type="text" />
        <button type="submit">Click Me!</button>
      </form>
      <section id="greeting">{greeting}</section>
    </main>
  );
}

export default App;
