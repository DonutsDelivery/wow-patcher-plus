import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import "./App.css";

function App() {
  return (
    <div className="min-h-screen bg-background text-foreground p-8">
      <h1 className="text-2xl font-bold mb-4">Turtle WoW HD Patcher</h1>
      <p className="text-muted-foreground mb-4">Setting up UI...</p>
      <Progress value={33} className="w-64 mb-4" />
      <Button>Test Button</Button>
    </div>
  );
}

export default App;
