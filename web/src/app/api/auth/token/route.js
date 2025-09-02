import { NextResponse } from 'next/server';
import { getServerSession } from 'next-auth';
import { authOptions } from '@/app/api/auth/[...nextauth]/route';
import connectDB from '@/../lib/mongodb';
import User from '@/../models/User';
import jwt from 'jsonwebtoken';

// Generate a token for the Rust client
export async function POST(request) {
  try {
    const session = await getServerSession(authOptions);
    
    if (!session) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    await connectDB();
    
    const user = await User.findOne({ email: session.user.email });
    if (!user) {
      return NextResponse.json({ error: 'User not found' }, { status: 404 });
    }

    // Generate a long-lived token for the Rust client
    const token = jwt.sign(
      { 
        userId: user._id, 
        email: user.email,
        type: 'rust-client'
      },
      process.env.JWT_SECRET,
      { expiresIn: '30d' } // 30 days
    );

    return NextResponse.json({ 
      token,
      message: 'Token generated successfully. Use this in your Rust client.'
    });
  } catch (error) {
    console.error('Error generating token:', error);
    return NextResponse.json({ error: 'Internal server error' }, { status: 500 });
  }
}
