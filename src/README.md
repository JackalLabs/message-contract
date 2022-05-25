
# JACKAL Messaging Contract
- [Introduction](#Introduction)
-  [Sections](#Sections)
    - [Init](#Init)
    - [Handle](#Handle)
        -  [InitAddress](#--InitAddress)
        -  [CreateViewingKey](#--CreateViewingKey)
        -  [SendMessage](#--SendMessage)
        -  [DeleteAllMessages](#--SendMessage)

     - [Query](#Query))  
        - [GetMessages](#--GetContents)

# Introduction
Contract implementation of JACKAL messaging system.

# Sections

## Init
This is for instantiating the contract.
|Name|Type|Description|                                                                                       
|--|--|--|
|prng_seed  | String  |  Pseudo Random Number Generator (PRNG) is a starting value to use for the generation of the pseudo random sequence.

## Handle 
### - InitAddress
For first time user. Create empty collection with a placeholder message and viewing_key
##### Request
|Name|Type|Description|                                                                                       
|--|--|--|
|entropy  | String  |  "entropy" is a term in physics, originally. In cryptography, it's usually used to talk about "source of randomness". 

##### Response
```json
{
  "data": {
    "key": "anubis_key_Th1s1sAn3xAMpl3+WfrGzBWrVdsh8="
  }
}
```

### - CreateViewingKey
**InitAddress** already create a viewing key for you when you first start using Jackal-messaging, but in case you want a new one, this will replace your current viewing key with a new one. 
##### Request
|Name|Type|Description|                                                                                       
|--|--|--|
|entropy  | String  |  "entropy" is a term in physics, originally. In cryptography, it's usually used to talk about "source of randomness". 
|padding  | String  |  "Padding is simply an optional parameter that can be used to obfuscate the length of the entropy string."

##### Response
```json
{
  "data": {
    "key": "anubis_key_Th1s1sAn3xAMpl3+WfrGzBWrVdsh8="
  }
}
```

### - SendMessage
Creates and sends a message to recipient. 

If recipient does not already have a collection:
    - initialize collection with placeholder message.
    - save message to collection.
    - recipient is responsible for a creating viewing key to view their messages. 

else:
    - save message to recipient's collection. 

##### Request
|Name|Type|Description|                                                                                       
|--|--|--|
|to  | String  |  "The recipient". 
|contents  | String  |  "A notification string, e.g., 'Sender has shared Pepe.jpg with you'"

### - DeleteAllMessages 

deletes all messages except for placeholder message 

n

## Queries

#### - GetMessages
Get all messages from a collection 

##### Request
|Name|Type|Description|                                                                                       
|--|--|--|
|behalf | String  | user address
|key    | String  | viewing key

##### Response

An array of messages 

```json
{
  "messages": [
      {
          "contents": "Hello: Sender has shared Pepe.jpg with you",
          "owner": "secret1j4jg2ahr7fp2uu9rfq5jrkhtychlharm6t5etx", 
      },
      {
          "contents": "Hello: Sender has shared Hasbullah.jpg with you",
          "owner": "secret1h7rvnn9lfs5507j9eazdxu4ewt7eg6hg2vgcrs", 
      }
  ]
}
```


