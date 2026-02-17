import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import 'katex/dist/katex.min.css'
import { InlineMath, BlockMath } from 'react-katex'
import './App.css'

const GROQ_API_KEY = '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'

// Render text with LaTeX support
const renderWithLatex = (text: string) => {
  // Split by LaTeX patterns: \(...\) for inline and \[...\] for block
  const parts = text.split(/(\\\(.*?\\\)|\\\[.*?\\\])/g)

  return parts.map((part, index) => {
    if (part.startsWith('\\(') && part.endsWith('\\)')) {
      // Inline math
      const math = part.slice(2, -2)
      return <InlineMath key={index} math={math} />
    } else if (part.startsWith('\\[') && part.endsWith('\\]')) {
      // Block math
      const math = part.slice(2, -2)
      return <BlockMath key={index} math={math} />
    }
    return <span key={index}>{part}</span>
  })
}

function App() {
  const [overlayVisible, setOverlayVisible] = useState(false)
  const [question, setQuestion] = useState('')
  const [response, setResponse] = useState('')
  const [loading, setLoading] = useState(false)

  const askAI = async () => {
    if (!question.trim()) return
    setLoading(true)
    setResponse('')
    try {
      const result = await invoke<string>('ask_ai', {
        question: question,
        apiKey: GROQ_API_KEY
      })
      setResponse(result)
    } catch (error) {
      setResponse(`Error: ${error}`)
    }
    setLoading(false)
  }

  const toggleOverlay = async () => {
    if (overlayVisible) {
      await invoke('hide_overlay_window')
      setOverlayVisible(false)
    } else {
      await invoke('create_overlay', { width: 320.0, height: 500.0 })
      // Set the Groq API key for the overlay chat
      await invoke('set_groq_api_key', { key: GROQ_API_KEY })
      setOverlayVisible(true)
    }
  }

  const closeOverlay = async () => {
    await invoke('close_overlay_window')
    setOverlayVisible(false)
  }

  return (
    <div className="app">
      <div className="container">
        <div className="header">
          <div className="logo">
            <span className="logo-icon">🐱</span>
          </div>
          <h1>Catpanion</h1>
          <p className="tagline">Your cute cat companion</p>
        </div>

        <div className="card">
          <div className="status">
            <div className={`status-indicator ${overlayVisible ? 'active' : ''}`} />
            <span>{overlayVisible ? 'Mittens is here!' : 'Mittens is sleeping...'}</span>
          </div>

          <button
            className={`primary-btn ${overlayVisible ? 'active' : ''}`}
            onClick={toggleOverlay}
          >
            {overlayVisible ? 'Hide Mittens' : 'Wake Mittens'}
          </button>

          {overlayVisible && (
            <button className="secondary-btn" onClick={closeOverlay}>
              Say Goodbye
            </button>
          )}
        </div>

        <div className="card gemini-card">
          <h3>Chat with Mittens</h3>
          <div className="gemini-input-row">
            <input
              type="text"
              value={question}
              onChange={(e) => setQuestion(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && askAI()}
              placeholder="Ask anything..."
              className="gemini-input"
              disabled={loading}
            />
            <button
              className="primary-btn gemini-btn"
              onClick={askAI}
              disabled={loading || !question.trim()}
            >
              {loading ? '...' : 'Ask'}
            </button>
          </div>
          {response && (
            <div className="gemini-response">
              {renderWithLatex(response)}
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

export default App
