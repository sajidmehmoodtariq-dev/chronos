import { NextResponse } from 'next/server';
import jwt from 'jsonwebtoken';
import connectDB from '@/../lib/mongodb';
import ActivityLog from '@/../models/ActivityLog';

export async function POST(request) {
  try {
    const authHeader = request.headers.get('authorization');
    if (!authHeader || !authHeader.startsWith('Bearer ')) {
      return NextResponse.json({ error: 'Missing or invalid authorization header' }, { status: 401 });
    }

    const token = authHeader.substring(7);
    let decoded;
    
    try {
      decoded = jwt.verify(token, process.env.JWT_SECRET);
    } catch (error) {
      return NextResponse.json({ error: 'Invalid token' }, { status: 401 });
    }

    if (decoded.type !== 'rust-client') {
      return NextResponse.json({ error: 'Invalid token type' }, { status: 401 });
    }

    const body = await request.json();
    const { logs } = body;

    if (!Array.isArray(logs)) {
      return NextResponse.json({ error: 'Logs must be an array' }, { status: 400 });
    }

    await connectDB();

    // Process and save logs
    const savedLogs = [];
    for (const logEntry of logs) {
      try {
        // Parse log entry and create structured data
        const log = await ActivityLog.create({
          userId: decoded.userId,
          timestamp: new Date(logEntry.timestamp),
          type: logEntry.type,
          data: logEntry.data
        });
        savedLogs.push(log);
      } catch (error) {
        console.error('Error saving log entry:', error);
      }
    }

    return NextResponse.json({ 
      message: 'Logs synced successfully',
      saved: savedLogs.length,
      total: logs.length
    });
  } catch (error) {
    console.error('Error syncing logs:', error);
    return NextResponse.json({ error: 'Internal server error' }, { status: 500 });
  }
}
