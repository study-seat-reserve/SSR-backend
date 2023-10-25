/*
座位表 (Seat)
seat_id (主鍵)
OtherInfo

用戶表 (Users)
user_id (主鍵)
user_name
Password_hash
email

預約表 (Reservations)
ReservationID (主鍵)
user_id (外鍵)
seat_id (外鍵)
Date (考慮到日期)
StartTime
EndTime


時段表 (TimeSlots)
Date (主鍵)
StartTime (例如: 09:00)
EndTime (例如: 09:30)
Availability (可預約或不可預約)

trigger?
*/
