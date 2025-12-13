# How to Create Your Own Server in the Cloud ☁️

This guide will walk you through setting up a small computer in the cloud (called an "EC2 instance") that you can connect to from your laptop.

---

## Step 1: Create an AWS Account

1. Go to [aws.amazon.com](https://aws.amazon.com)
2. Click **Create an AWS Account**
3. Follow the steps (you'll need an email, phone number, and a credit/debit card)
4. Choose the **Free Tier** — you won't be charged for small servers for 12 months

---

## Step 2: Go to the EC2 Dashboard

1. Once logged in, look for the search bar at the top
2. Type **EC2** and click on it
3. You're now in the EC2 Dashboard — this is where you manage your servers

---

## Step 3: Change to the Australia Region

The "region" is where your server lives physically. Let's put it in Australia.

1. Look at the **top right** of the screen (near your account name)
2. You'll see a region name like "N. Virginia" or "Ohio"
3. Click on it and select **Asia Pacific (Sydney)** — this is `ap-southeast-2`

---

## Step 4: Create Your SSH Key (Your Secret Password File)

An SSH key is like a special password file that lets you log into your server securely.

1. On the left menu, click **Key Pairs** (under "Network & Security")
2. Click the orange **Create key pair** button
3. Give it a name like `my-key`
4. For **Key pair type**, leave it as **RSA**
5. For **Private key file format**:
   - If you're on **Mac or Linux**: choose `.pem`
   - If you're on **Windows**: choose `.ppk` if using PuTTY, or `.pem` if using Windows Terminal
6. Click **Create key pair**
7. A file will download — **keep this file safe!** You can't download it again.

### Move your key to a safe spot

**On Mac/Linux**, open Terminal and run:
```bash
mv ~/Downloads/my-key.pem ~/.ssh/
chmod 400 ~/.ssh/my-key.pem
```

**On Windows**, just remember where you saved it (like your Downloads folder).

---

## Step 5: Launch Your Server

1. On the left menu, click **Instances**
2. Click the orange **Launch instances** button

Now fill in these settings:

### Name
- Give it a friendly name like `my-first-server`

### Application and OS Image
- Click **Ubuntu** (it's free and beginner-friendly)
- Leave the default Ubuntu version selected

### Instance type
- Select **t2.micro** — this is the cheapest and is FREE for 12 months
- It says "Free tier eligible" next to it

### Key pair
- Select the key you created earlier (`my-key`)

### Network settings
- Click **Edit**
- Make sure **Auto-assign public IP** is set to **Enable**
- Under **Firewall (security groups)**, select **Create security group**
- Tick the box for **Allow SSH traffic from** and leave it as "Anywhere" (0.0.0.0/0)

### Storage
- The default 8 GB is fine (and free)

### Launch it!
- Click the orange **Launch instance** button at the bottom
- Click **View all instances**

---

## Step 6: Wait for Your Server to Start

1. You'll see your instance in the list
2. Wait until **Instance state** shows **Running** (green)
3. Wait until **Status check** shows **2/2 checks passed**

This takes about 1-2 minutes.

---

## Step 7: Find Your Server's Address

1. Click on your instance (click the Instance ID link)
2. Look for **Public IPv4 address** — it'll look something like `13.55.123.45`
3. Copy this address

---

## Step 8: Connect to Your Server! 🎉

### On Mac or Linux

1. Open **Terminal**
2. Type this command (replace the IP with yours):

```bash
ssh -i ~/.ssh/my-key.pem ubuntu@YOUR-IP-ADDRESS
```

For example:
```bash
ssh -i ~/.ssh/my-key.pem ubuntu@13.55.123.45
```

3. If it asks "Are you sure you want to continue connecting?" type `yes`
4. You're in! You should see something like `ubuntu@ip-172-31-xx-xx:~$`

### On Windows (using Windows Terminal or PowerShell)

1. Open **Windows Terminal** or **PowerShell**
2. Type this command (replace the path and IP):

```powershell
ssh -i C:\Users\YourName\Downloads\my-key.pem ubuntu@YOUR-IP-ADDRESS
```

---

## Quick Reference

| What | Value |
|------|-------|
| Region | Asia Pacific (Sydney) `ap-southeast-2` |
| Instance type | t2.micro (free tier) |
| Operating system | Ubuntu |
| Username for SSH | `ubuntu` |
| SSH command | `ssh -i ~/.ssh/my-key.pem ubuntu@YOUR-IP` |

---

## Troubleshooting

**"Permission denied"** — Your key file permissions might be wrong. Run:
```bash
chmod 400 ~/.ssh/my-key.pem
```

**"Connection timed out"** — Check that your security group allows SSH (port 22) and that you have a public IP.

**Can't find your key file** — Check your Downloads folder. You can only download it once when you create it!
