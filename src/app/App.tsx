import { BrowserRouter } from 'react-router-dom'

function App() {
  return (
    <BrowserRouter>
      <div className="min-h-screen bg-gray-50">
        <header className="bg-white shadow">
          <div className="max-w-7xl mx-auto px-4 py-4">
            <h1 className="text-2xl font-bold text-gray-900">School Fees System</h1>
          </div>
        </header>
        <main className="max-w-7xl mx-auto px-4 py-6">
          <p className="text-gray-600">Welcome to School Fees System</p>
        </main>
      </div>
    </BrowserRouter>
  )
}

export default App
