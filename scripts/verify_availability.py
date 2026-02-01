import requests
import json
import time
import random
import string

BASE_URL = "http://localhost:3000/api"

def random_string(length=10):
    return ''.join(random.choices(string.ascii_lowercase, k=length))

def run():
    session = requests.Session()
    
    # 0. Admin Login
    print("Logging in as Admin...")
    admin_login_res = session.post(f"{BASE_URL}/admin/login", json={
        "email": "admin@example.com",
        "password": "password123" 
    })
    
    if admin_login_res.status_code != 200:
        print(f"Admin login failed: {admin_login_res.text}")
        return
        
    admin_token = admin_login_res.json()['token']
    print(f"Admin Access Token: {admin_token[:10]}...")

    # 1. Register User (DM)
    dm_email = f"dm_{random_string()}@test.com"
    dm_pass = "Password123!"
    print(f"Registering DM: {dm_email}...")
    
    reg_res = session.post(f"{BASE_URL}/auth/register", json={
        "email": dm_email,
        "password": dm_pass,
        "name": "Dungeon Master"
    })
    
    if reg_res.status_code != 201:
        print(f"Registration failed: {reg_res.text}")
        return
        
    user_data = reg_res.json()
    user_id = user_data['user']['id']
    user_token = user_data['token']
    print(f"User registered: {user_id}")
    
    # 2. Promote to DM
    print("Promoting user to DM...")
    promote_res = session.put(
        f"{BASE_URL}/admin/users/{user_id}/role",
        headers={"Authorization": f"Bearer {admin_token}"},
        json={"role": "dm"}
    )
    
    if promote_res.status_code != 200:
        print(f"Promotion failed: {promote_res.text}")
        return
    print("User promoted to DM")

    # 3. Create Poll
    print("Creating poll...")
    poll_res = session.post(
        f"{BASE_URL}/polls", 
        headers={"Authorization": f"Bearer {user_token}"},
        json={
            "title": f"Vecna Test {random_string(4)}",
            "description": "Test poll",
            "location": "Discord",
            "dates": ["2026-01-01", "2026-01-02"],
            "participants": [f"p1_{random_string()}@test.com", f"p2_{random_string()}@test.com"]
        }
    )
    
    if poll_res.status_code != 200:
        print(f"Failed to create poll: {poll_res.text}")
        return

    poll_data = poll_res.json()
    poll_id = poll_data['id']
    print(f"Poll created: {poll_id}")
    
    # Need to fetch poll details to get participant tokens? 
    # Usually Join gives token.
    # The creator is NOT automatically a participant in the current logic unless added.
    
    # 4. Join Poll as Player
    print("Joining poll as Player 1...")
    p1_email = f"player1_{random_string()}@test.com"
    join_res = session.post(f"{BASE_URL}/polls/{poll_id}/join", json={
        "name": "Player One",
        "email": p1_email
    })
    
    if join_res.status_code != 200:
        print(f"Failed to join: {join_res.text}")
        return
        
    player_data = join_res.json()
    player_id = player_data['id']
    access_token = player_data['access_token']
    print(f"Joined as {player_id}")

    # 5. Submit Availability
    print("Submitting availability...")
    avail_res = session.post(
        f"{BASE_URL}/polls/{poll_id}/participants/{player_id}/availability", 
        json={
            "access_token": access_token,
            "availability": [
                {"date": "2026-01-01", "time_slot": "18:00", "status": "available"}
            ]
        }
    )
    
    if avail_res.status_code != 200:
        print(f"Failed to submit availability: {avail_res.text}")
        return
    print("Availability submitted")

    # 6. Fetch Poll Details
    print("Fetching poll details...")
    # Fetching poll details likely doesn't require auth or maybe it does?
    # handlers::get_poll doesn't verify AuthUser? Let's assume public for now or use user_token.
    detail_res = session.get(f"{BASE_URL}/polls/{poll_id}")
    
    if detail_res.status_code != 200:
        print(f"Failed to fetch details: {detail_res.text}")
        return
        
    details = detail_res.json()
    availability = details['availability']
    print(f"Availability records: {len(availability)}")
    print(json.dumps(availability, indent=2))
    
    if len(availability) > 0:
        print("SUCCESS: Availability is present.")
    else:
        print("FAILURE: Availability is missing.")

if __name__ == "__main__":
    try:
        run()
    except Exception as e:
        print(f"Error: {e}")
