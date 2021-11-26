
## pipeline


```json
[{$lookup: {
 from: 'events',
 localField: 'id',
 foreignField: 'user_id',
 as: 'event'
}}, {$unwind: '$event'}, {$match: {
 'event.event_id': 1
}}, {$project: {
 _id: 0,
 name: 1,
 username: 1,
 'event.event_id': 1,
 'event.user_id': 1,
 'event.point': 1
}}, {$group: {
 _id: '$name',
 star: {
  $sum: '$event.event_id'
 },
 point: {
  $sum: '$event.point'
 },
 total: {
  $sum: 1
 }
}}, {$sort: {
 point: 1
}}]
```
