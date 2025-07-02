// src/components/RelatedNotes.jsx
import React from 'react';
import formatTime from '../utils/formatTime';

export default function RelatedNotes({ relatedPosts, loadNode }) {
  return (
    <div style={{ marginTop: 20 }}>
      <h3 style={{ color: '#fff', margin: 0, marginBottom: 10 }}>관련 노트</h3>
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: 10, textAlign: 'center' }}>
        {relatedPosts.map(rp => (
          <div key={rp.id}
            style={{
              background: '#2e2e2e', padding: 12, borderRadius: 4, cursor: 'pointer',
              display: 'flex', flexDirection: 'column', justifyContent: 'center', alignItems: 'center'
            }}
            onClick={() => loadNode({ postId: rp.id })}
          >
            <div style={{
              color: '#fff', fontWeight: 'bold', marginBottom: 4,
              overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap'
            }}>
              {rp.title}
            </div>
            <div style={{ color: '#888', fontSize: 12 }}>
              {formatTime(rp.updated_at)}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
